use petgraph::{
    algo::toposort,
    data::FromElements,
    graph::{DiGraph, NodeIndex},
};
use thiserror::Error;

use crate::{commands::command_buffer::CommandBuffer, world::World};

use super::system_config::{IntoSystemConfigs, SystemConfig, SystemConfigs, SystemMeta};

#[derive(Debug, Error)]
pub enum SystemGraphError {
    #[error("Dependencies not satisfied: {0:?}")]
    DependencyNotSatisfied(String),
}

#[derive(Clone, Debug)]
struct FailedDependencies {
    system_name: &'static str,
    before: Vec<&'static str>,
    after: Vec<&'static str>,
}

pub type SystemGraphResult<T> = Result<T, SystemGraphError>;

pub type DependencyGraph = DiGraph<NodeId, (), usize>;

pub struct SystemGraph {
    systems: Vec<SystemConfig>,
    cached_toposort: Option<Vec<usize>>,
}

fn root_system() {}

impl SystemGraph {
    pub fn new() -> Self {
        let root_system_config =
            if let SystemConfigs::Config(config) = root_system.into_system_configs() {
                config
            } else {
                unreachable!()
            };
        Self {
            systems: vec![root_system_config],
            cached_toposort: None,
        }
    }

    pub fn add_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) {
        let systems = systems.into_system_configs();

        match systems {
            SystemConfigs::Config(config) => self.systems.push(config),
            SystemConfigs::Configs(configs) => {
                configs.into_iter().for_each(|s| self.add_systems(s))
            }
        }
    }

    pub fn init_systems(&mut self, world: &mut World) {
        self.systems
            .iter_mut()
            .filter(|s| !s.system.is_init())
            .for_each(|s| s.system.init(unsafe { world.as_unsafe_cell() }));

        if self.cached_toposort.is_none() {
            let toposort = build_dependency_graph(&self.systems)
                .1
                .into_iter()
                .map(|index| index.index())
                .collect();

            self.cached_toposort = Some(toposort);
        }
    }

    pub fn run_systems(&mut self, world: &mut World) -> CommandBuffer {
        if let Some(toposort) = self.cached_toposort.as_ref() {
            toposort
                .iter()
                .map(|i| &raw mut self.systems[*i].system)
                .filter_map(|s| {
                    let system = unsafe { &mut **s };
                    if system.is_init() {
                        system.run(unsafe { world.as_unsafe_cell() });
                        system.take_deferred()
                    } else {
                        None
                    }
                })
                .fold(CommandBuffer::new(), |mut acc, mut curr| {
                    acc.append(&mut curr);
                    acc
                })
        } else {
            panic!("Trying to execute system graph without initializing systems in it!");
        }
    }

    pub fn dependency_graph(&self) -> (DependencyGraph, Vec<NodeIndex<usize>>) {
        build_dependency_graph(&self.systems)
    }
}

impl Default for SystemGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct NodeId(usize, &'static str);

impl From<NodeId> for NodeIndex<usize> {
    fn from(value: NodeId) -> Self {
        Self::new(value.0)
    }
}

fn build_dependency_graph(
    systems: &[SystemConfig],
) -> (DiGraph<NodeId, (), usize>, Vec<NodeIndex<usize>>) {
    let mut en = systems
        .iter()
        .filter_map(|sys| {
            if sys.system.is_init() {
                Some(sys.meta.clone())
            } else {
                None
            }
        })
        .enumerate();

    let mut dag = DiGraph::from_elements(
        en.clone()
            .map(|(i, meta)| {
                let node = NodeId(i, meta.name);
                petgraph::data::Element::Node { weight: node }
            })
            .collect::<Vec<_>>(),
    );

    let mut skipped_root = en.clone();
    let root = skipped_root.next().unwrap();
    let skipped_root = skipped_root;

    let mut edges = skipped_root
        .clone()
        .clone()
        .map(|(i, meta)| {
            skipped_root
                .clone()
                .filter_map(|(n, dep_meta)| {
                    if dep_meta.before.contains(&meta.name) {
                        Some((NodeId(n, dep_meta.name), NodeId(i, meta.name)))
                    } else if dep_meta.after.contains(&meta.name) {
                        Some((NodeId(i, meta.name), NodeId(n, dep_meta.name)))
                    } else {
                        None
                    }
                })
                .chain(vec![(NodeId(0, root.1.name), NodeId(i, meta.name))])
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    edges.sort();
    edges.dedup();

    let reduced = edges.iter().filter(|(NodeId(x, _), NodeId(y, _))| {
        for (z, _) in en.clone() {
            let has_xz = edges
                .iter()
                .find(|(NodeId(maybe_x, _), NodeId(maybe_z, _))| maybe_x == x && *maybe_z == z)
                .is_some();

            let has_zy = edges
                .iter()
                .find(|(NodeId(maybe_z, _), NodeId(maybe_y, _))| *maybe_z == z && maybe_y == y)
                .is_some();

            if has_xz && has_zy {
                return false;
            }
        }

        true
    });

    dag.extend_with_edges(reduced);

    let toposort = toposort(&dag, None).unwrap();

    (dag, toposort)
}
