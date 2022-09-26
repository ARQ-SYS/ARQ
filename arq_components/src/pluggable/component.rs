use std::{mem, any::Any};
use rocket::Route;


/// A trait that represents a component that can be mounted to the ARQ_CORE.
/// This trait is used when one wants to add certain functionality to the ARQ_CORE.
/// Note: Managed state is not *YET* supported.
pub trait Component: Any + Send + Sync {

    /// This function should return the name of the component.
    fn name(&self) -> &'static str;
    /// This function is fired before the component is mounted by CORE.
    fn on_component_load(&self) {}
    /// This function is fired after the component is unmounted by CORE.
    /// This happens when the CORE is shutting down.
    fn on_component_unload(&self) {}
    /// This function should return the routes that should be mounted by CORE.
    /// The returning should be handled by the `arq_components::pluggable::component::ComponentFactory`
    fn routes(&self) -> (*mut Route, usize, usize);
}

/// This struct is used to export the component's routes to the CORE.
/// It must be used alongside the macro `declare_component!`, so the ComponentManager can find the Component.
pub struct ComponentFactory {
    pub routes: Vec<Route>
}

impl ComponentFactory {
    /// Constructs a new ComponentFactory.
    pub fn new() -> Self {
        ComponentFactory {
            routes: Vec::new()
        }
    }
    /// Adds a route to the ComponentFactory.
    pub fn add_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }
    /// Adds multiple routes to the ComponentFactory.
    pub fn add_routes(mut self, routes: Vec<Route>) -> Self {
        self.routes.extend(routes);
        self
    }
    /// This returns the routes that should be mounted by CORE.
    /// This consumes the ComponentFactory.
    pub fn export(mut self) -> (*mut Route, usize, usize) {
        // This makes sure that the routes' length and capacity are the same.
        self.routes.shrink_to_fit();
        assert!(self.routes.len() == self.routes.capacity());

        let ptr = self.routes.as_mut_ptr();
        let len = self.routes.len();
        // This makes sure that the routes are not dropped.
        mem::forget(self.routes);

        (ptr, len, len)
    }
}