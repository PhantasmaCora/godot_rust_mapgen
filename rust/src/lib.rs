use godot::prelude::*;

mod nodes;
mod resource;
mod datagrid;
mod button_plugin;
mod algorithm;


struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
