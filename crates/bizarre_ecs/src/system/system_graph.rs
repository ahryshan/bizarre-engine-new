use std::{
    collections::{HashMap, VecDeque},
    iter::Iterator,
    ptr::NonNull,
};

use bizarre_memory::arena::typed::TypedArena;

use crate::World;

use super::{
    error::{SystemError, SystemResult},
    StoredSystem,
};

type NodePtr = NonNull<GraphNode<SystemData>>;

pub struct GraphNode<T> {
    data: T,
    parent: Option<NonNull<Self>>,
    children: Vec<NonNull<Self>>,
}

impl<T> GraphNode<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            parent: None,
            children: Vec::default(),
        }
    }

    pub fn add_child(&mut self, child: NonNull<Self>) {
        self.children.push(child);
    }

    pub fn set_parent(&mut self, parent: NonNull<Self>) {
        self.parent = Some(parent)
    }
}

pub struct SystemGraph {
    root: Option<NodePtr>,
    arena: TypedArena<GraphNode<SystemData>>,
    deps_map: HashMap<NodePtr, Box<[&'static str]>>,
}

impl SystemGraph {
    pub fn new() -> Self {
        Self {
            root: None,
            arena: TypedArena::new(256),
            deps_map: HashMap::default(),
        }
    }

    pub fn add_system(
        &mut self,
        system: StoredSystem,
        name: &'static str,
        deps: &[&'static str],
    ) -> SystemResult {
        let system_data = SystemData {
            deps: deps.into(),
            name,
            system,
            init: false,
        };
        let mut node = self.arena.alloc(GraphNode::new(system_data));

        match &mut self.root {
            Some(root) if deps.is_empty() => {
                unsafe {
                    root.as_mut().add_child(node);
                }
                Ok(())
            }
            Some(_) => {
                let mut parent = self.find_first_appropriate_parent(name, deps)?;
                unsafe {
                    parent.as_mut().add_child(node);
                    node.as_mut().set_parent(parent);
                    self.deps_map.insert(node, gather_parent_names(node).into());
                }
                Ok(())
            }
            None if deps.is_empty() => {
                self.root = Some(node);
                self.deps_map.insert(node, [name].into());
                Ok(())
            }
            None => Err(SystemError::NoDependency {
                system_name: name,
                not_found: deps.join(", "),
            }),
        }
    }

    pub fn init_systems(&self, world: &World) {
        if self.root.is_none() {
            return;
        }

        NodeIterator::new(self.root.unwrap())
            .filter_map(|mut n| {
                let node = unsafe { n.as_mut() };
                if node.data.init {
                    None
                } else {
                    node.data.init = true;
                    Some(&mut node.data.system)
                }
            })
            .for_each(|s| s.init(world));
    }

    pub fn run_systems(&self, world: &World) {
        if self.root.is_none() {
            return;
        }

        NodeIterator::new(self.root.unwrap())
            .filter(|n| {
                let system = unsafe { n.as_ref() };
                system.data.init
            })
            .for_each(|mut n| {
                let system = unsafe { &mut n.as_mut().data };
                system.system.run(world)
            });
    }

    fn find_first_appropriate_parent(
        &self,
        name: &'static str,
        deps: &[&'static str],
    ) -> SystemResult<NodePtr> {
        if self.root.is_none() {
            return Err(SystemError::NoDependency {
                system_name: name,
                not_found: deps.join(", "),
            });
        }

        let mut best_not_found: Option<Vec<&'static str>> = None;
        let mut node = None;

        for (ptr, map_deps) in self.deps_map.iter() {
            let not_found = deps
                .iter()
                .copied()
                .filter(|d| !map_deps.contains(d))
                .collect::<Vec<_>>();

            if not_found.is_empty() {
                node = Some(*ptr);
                break;
            }

            if best_not_found.is_none()
                || (best_not_found.is_some()
                    && not_found.len() < best_not_found.as_ref().unwrap().len())
            {
                best_not_found = Some(not_found);
            }
        }

        node.ok_or_else(|| {
            let not_found = best_not_found.unwrap().join(", ");
            SystemError::NoDependency {
                system_name: name,
                not_found,
            }
        })
    }
}

unsafe fn gather_parent_names(mut node: NodePtr) -> Vec<&'static str> {
    let node = node.as_mut();

    match node.parent {
        Some(parent) => {
            let mut ret = vec![node.data.name];
            ret.append(&mut gather_parent_names(parent));
            ret
        }
        None => vec![node.data.name],
    }
}

impl Default for SystemGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SystemData {
    name: &'static str,
    deps: Box<[&'static str]>,
    system: StoredSystem,
    init: bool,
}

struct NodeIterator {
    queue: VecDeque<NodePtr>,
}

impl NodeIterator {
    pub fn new(node: NodePtr) -> Self {
        Self {
            queue: VecDeque::from(vec![node]),
        }
    }
}

impl Iterator for NodeIterator {
    type Item = NodePtr;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ptr) = self.queue.pop_front() {
            let child_iter = unsafe { ptr.as_ref().children.iter() };
            self.queue.extend(child_iter);

            Some(ptr)
        } else {
            None
        }
    }
}
