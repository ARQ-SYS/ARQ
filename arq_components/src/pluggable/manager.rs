use std::ffi::OsStr;

use libloading::{Library, Symbol};
use rocket::Route;

#[cfg(feature = "broken")]
use rocket::fairing::Fairing;

use tracing::{info, debug};

use crate::pluggable::{middleware::MiddlewareComponent, component::Component};

use anyhow::{Result, Context};


/// This struct is used to orchestrate the loading of the components and middlewares.
/// It will be used by the CORE to load the components and middlewares.
pub struct ComponentManager {
    components: Vec<Box<dyn Component>>,
    middlewares: Vec<Box<dyn MiddlewareComponent>>,
    loaded_libs: Vec<Library>
}

impl ComponentManager {
    /// Constructs a new ComponentManager.
    pub fn new() -> Self {
        ComponentManager {
            components: Vec::new(),
            middlewares: Vec::new(),
            loaded_libs: Vec::new()
        }
    }
    /// Loads the component from the given path.
    pub unsafe fn load_components<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<()> {

        type ComponentConstructor = fn() -> *mut dyn Component;

        debug!("Loading component from {}", filename.as_ref().to_string_lossy());
        let lib = Library::new(filename.as_ref()).context("Failed to load library")?;

        self.loaded_libs.push(lib);               
        let lib = self.loaded_libs.last().unwrap(); // This is safe because we just pushed it.

        let component_constructor: Symbol<ComponentConstructor> = lib.get(b"_arq_component_constructor").context("Unable to locate symbol. Please make sure that you're exporting it with declare_component!() macro")?;
        let raw = component_constructor();
        let component = Box::from_raw(raw);
        debug!("Loaded component: {}", component.name());
        component.on_component_load();
        self.components.push(component);

        Ok(())
    }

    /// This functions unloads the components and middlewares from ComponentManager.
    /// This wont unload the components and middlewares from the CORE, when it's already running.
    pub fn unload(&mut self) {
        info!("Unloading middleware");
        for middleware in self.middlewares.drain(..) {
            debug!("Unloading middleware: {}", middleware.name());
            middleware.on_middleware_unload();
        }
        
        info!("Unloading components");
        for component in self.components.drain(..) {
            debug!("Unloading middleware: {}", component.name());
            component.on_component_unload();
        }

        for lib in self.loaded_libs.drain(..) {
            drop(lib)
        }
    }
    /// This function returns the routes that should be mounted by CORE.
    pub fn get_routes(&self) -> Vec<Route> {

        let mut out = Vec::new();
        for comp in &self.components {
            let raw = comp.routes();
            unsafe {
                let complete = Vec::from_raw_parts(raw.0, raw.1, raw.2);
                out.extend(complete);
            }
        }
        return out;
    }

}


#[allow(unused)]
#[cfg(feature = "broken")]
impl ComponentManager {


    
    pub fn get_middleware(&self) -> Vec<Box<dyn Fairing>> {
        let mut out = Vec::new();
        for middleware in &self.middlewares {
            let raw = middleware.middlewares();
            unsafe {
                // let complete = Box::from(Vec::from_raw_parts(raw.0, raw.1, raw.2));
                // out.extend(complete);
            }
        }
        return out;
    }

    // Loads the middleware from the given path.
    pub unsafe fn load_middleware<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<()> {

        type MiddlewareConstructor = fn() -> *mut dyn MiddlewareComponent;

        debug!("Loading middleware from {}", filename.as_ref().to_string_lossy());
        let lib = Library::new(filename.as_ref()).context("Failed to load library")?;

        self.loaded_libs.push(lib);               
        let lib = self.loaded_libs.last().unwrap(); // This is safe because we just pushed it.

        let middleware_constructor: Symbol<MiddlewareConstructor> = lib.get(b"__arq_middleware_constructor").context("Unable to locate symbol. Please make sure that you're exporting it with declare_component!() macro")?;
        let raw = middleware_constructor();
        let middleware = Box::from_raw(raw);
        debug!("Loaded middleware: {}", middleware.name());
        middleware.on_middleware_load();
        self.middlewares.push(middleware);

        Ok(())
    }


}
