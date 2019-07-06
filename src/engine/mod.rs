use crate::scene::*;
use crate::utils::pool::*;
use crate::renderer::renderer::*;
use crate::resource::*;
use std::path::*;
use crate::resource::texture::*;
use serde::{Serialize, Deserialize};
use crate::utils::rcpool::{RcPool, RcHandle};
use std::collections::VecDeque;
use crate::renderer::surface::SurfaceSharedData;
use crate::resource::model::Model;

pub struct ResourceManager {
    resources: RcPool<Resource>,
    /// Path to textures, extensively used for resource files
    /// which stores path in weird format (either relative or absolute) which
    /// is obviously not good for engine.
    textures_path: PathBuf,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            resources: RcPool::new(),
            textures_path: PathBuf::from("data/textures/"),
        }
    }
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager::default()
    }

    #[inline]
    pub fn for_each_texture_mut<Func>(&mut self, mut func: Func) where Func: FnMut(&mut Texture) {
        for resource in self.resources.iter_mut() {
            if let ResourceKind::Texture(texture) = resource.borrow_kind_mut() {
                func(texture);
            }
        }
    }

    #[inline]
    fn add_resource(&mut self, resource: Resource) -> RcHandle<Resource> {
        self.resources.spawn(resource)
    }

    /// Searches for a resource of specified path, if found - returns handle to resource
    /// and increases reference count of resource.
    #[inline]
    fn find_resource(&mut self, path: &Path) -> RcHandle<Resource> {
        for i in 0..self.resources.get_capacity() {
            if let Some(resource) = self.resources.at(i) {
                if resource.get_path() == path {
                    return self.resources.handle_from_index(i);
                }
            }
        }
        RcHandle::none()
    }

    #[inline]
    pub fn borrow_resource(&self, resource_handle: &RcHandle<Resource>) -> Option<&Resource> {
        self.resources.borrow(resource_handle)
    }

    #[inline]
    pub fn borrow_resource_mut(&mut self, resource_handle: &RcHandle<Resource>) -> Option<&mut Resource> {
        self.resources.borrow_mut(resource_handle)
    }

    #[inline]
    pub fn share_resource_handle(&self, resource_handle: &RcHandle<Resource>) -> RcHandle<Resource> {
        self.resources.share_handle(resource_handle)
    }

    #[inline]
    pub fn get_textures_path(&self) -> &Path {
        self.textures_path.as_path()
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    scenes: Pool<Scene>,
    surf_data_storage: RcPool<SurfaceSharedData>,

    #[serde(skip)]
    resource_manager: ResourceManager,
}

impl State {
    #[inline]
    pub fn new() -> Self {
        State {
            scenes: Pool::new(),
            resource_manager: ResourceManager::new(),
            surf_data_storage: RcPool::new(),
        }
    }

    /// Returns handle of existing resource, or if resource is not loaded yet,
    /// loads it and returns it handle. If resource could not be loaded, returns
    /// none handle.
    pub fn request_resource(&mut self, path: &Path) -> RcHandle<Resource> {
        let mut resource_handle = self.resource_manager.find_resource(path);

        if resource_handle.is_none() {
            // No such resource, try to load it.
            let extension = path.extension().
                and_then(|os| os.to_str()).
                map_or(String::from(""), |s| s.to_ascii_lowercase());

            resource_handle = match extension.as_str() {
                "jpg" | "jpeg" | "png" | "tif" | "tiff" | "tga" | "bmp" => {
                    match Texture::load(path) {
                        Ok(texture) => {
                            self.resource_manager.add_resource(Resource::new(path, ResourceKind::Texture(texture)))
                        }
                        Err(_) => {
                            println!("Unable to load texture!");
                            RcHandle::none()
                        }
                    }
                }
                "fbx" => {
                    match Model::load(path, self) {
                        Ok(model) => {
                            self.resource_manager.add_resource(Resource::new(path, ResourceKind::Model(model)))
                        }
                        Err(_) => {
                            println!("Unable to load model!");
                            RcHandle::none()
                        }
                    }
                }
                _ => {
                    println!("Unknown resource type!");
                    RcHandle::none()
                }
            }
        }

        resource_handle
    }

    #[inline]
    pub fn get_scenes(&self) -> &Pool<Scene> {
        &self.scenes
    }

    #[inline]
    pub fn get_scenes_mut(&mut self) -> &mut Pool<Scene> {
        &mut self.scenes
    }

    #[inline]
    pub fn get_surface_data_storage(&self) -> &RcPool<SurfaceSharedData> {
        &self.surf_data_storage
    }

    #[inline]
    pub fn get_resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    #[inline]
    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    #[inline]
    pub fn get_surface_data_storage_mut(&mut self) -> &mut RcPool<SurfaceSharedData> {
        &mut self.surf_data_storage
    }

    #[inline]
    pub fn add_scene(&mut self, scene: Scene) -> Handle<Scene> {
        self.scenes.spawn(scene)
    }

    #[inline]
    pub fn get_scene(&self, handle: &Handle<Scene>) -> Option<&Scene> {
        if let Some(scene) = self.scenes.borrow(handle) {
            return Some(scene);
        }
        None
    }

    #[inline]
    pub fn get_scene_mut(&mut self, handle: &Handle<Scene>) -> Option<&mut Scene> {
        if let Some(scene) = self.scenes.borrow_mut(handle) {
            return Some(scene);
        }
        None
    }
}

pub struct Engine {
    renderer: Renderer,
    state: State,
    events: VecDeque<glutin::Event>,
    running: bool,
}

impl Engine {
    #[inline]
    pub fn new() -> Engine {
        Engine {
            state: State::new(),
            renderer: Renderer::new(),
            events: VecDeque::new(),
            running: true,
        }
    }

    #[inline]
    pub fn get_state(&self) -> &State {
        &self.state
    }

    #[inline]
    pub fn get_state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update(&mut self, dt: f64) {
        let client_size = self.renderer.context.get_inner_size().unwrap();
        let aspect_ratio = (client_size.width / client_size.height) as f32;

        for scene in self.state.scenes.iter_mut() {
            scene.update(aspect_ratio, dt);
        }
    }

    pub fn poll_events(&mut self) {
        // Gather events
        let events = &mut self.events;
        events.clear();
        self.renderer.events_loop.poll_events(|event| {
            events.push_back(event);
        });
    }

    pub fn render(&mut self) {
        self.renderer.upload_resources(&mut self.state);
        self.renderer.render(&self.state);
    }

    #[inline]
    pub fn stop(&mut self) {
        self.running = false;
    }

    #[inline]
    pub fn pop_event(&mut self) -> Option<glutin::Event> {
        self.events.pop_front()
    }
}