<p align="center">
  <img src="https://github.com/nightingazer/bizarre-engine-rust/assets/80068087/1a2685a4-7609-409d-b07c-fb6d39552e32">
</p>

# Bizarre Engine

This is a game engine I'm making from scratch in Rust. I don't have a lot of experience in game dev
and I definitely have even less experience in game engine building. It will definitely be a very
strange, pretty possibly obnoxious in some aspects, and overall bizarre piece of
software.

With this project I'm trying to keep my dependency list to be as lean as possible while making as much as reasonable from scratch. Some of the dependencies will be replaced by my own implementation
as I gain experience and knowledge along the way.

I'm not considering adding DX, Metal or OpenGL support.

## Rodemap

- [x] ECS
- [x] Event system (I'm currently merging it with my ECS)
- [ ] Input handling
  - [ ] Rebindable controls
- [ ] Vulkan renderer (Will be repurpoused from the previous iteration)
  - [ ] Mesh loading
  - [ ] Runtime shader compilation
  - [ ] Multithreaded logging system with configurable loggers
- [ ] Physics system
- [ ] Hot reload for game code and assets
- [ ] Audio system

