use godot::prelude::*;
use godot::classes::{EditorPlugin, IEditorPlugin, EditorSelection, Button, editor_plugin::CustomControlContainer};

use crate::nodes::GeneratedGridMap;

#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct ButtonsPlugin {
    base: Base<EditorPlugin>,
    gen_button: Option<Gd<Button>>,
    pla_button: Option<Gd<Button>>,
    clr_button: Option<Gd<Button>>,
}

impl ButtonsPlugin {
    fn check_selection(&mut self) {

        if self.gen_button == None || self.pla_button == None {
            return;
        }

        let mut selection : Gd<EditorSelection>;

        {
            let interface = self.base_mut().get_editor_interface();
            if interface == None {
                return;
            }
            let interface = interface.unwrap();

            let oselection = interface.get_selection();
            if oselection == None {
                return;
            }
            selection = oselection.unwrap();
        }

        if selection.get_selected_nodes().len() != 1 {
            self.gen_button.as_mut().unwrap().hide();
            return;
        }

        let selected = selection.get_selected_nodes().at(0);

        if let Ok(_) = selected.try_cast::<GeneratedGridMap>() {
            self.gen_button.as_mut().unwrap().show();
            self.pla_button.as_mut().unwrap().show();
            self.clr_button.as_mut().unwrap().show();
        } else {
            self.gen_button.as_mut().unwrap().hide();
            self.pla_button.as_mut().unwrap().hide();
            self.clr_button.as_mut().unwrap().hide();
        }
    }

    fn on_gen_button_press(&mut self) {

        let mut selection : Gd<EditorSelection>;

        {
            let interface = self.base_mut().get_editor_interface();
            if interface == None {
                return;
            }
            let interface = interface.unwrap();

            let oselection = interface.get_selection();
            if oselection == None {
                return;
            }
            selection = oselection.unwrap();
        }

        if selection.get_selected_nodes().len() != 1 {
            return;
        }

        let selected = selection.get_selected_nodes().at(0);

        if let Ok(mut ggm) = selected.try_cast::<GeneratedGridMap>() {
            ggm.bind_mut().generate_default();
        }
    }

    fn on_pla_button_press(&mut self) {

        let mut selection : Gd<EditorSelection>;

        {
            let interface = self.base_mut().get_editor_interface();
            if interface == None {
                return;
            }
            let interface = interface.unwrap();

            let oselection = interface.get_selection();
            if oselection == None {
                return;
            }
            selection = oselection.unwrap();
        }

        if selection.get_selected_nodes().len() != 1 {
            return;
        }

        let selected = selection.get_selected_nodes().at(0);

        if let Ok(mut ggm) = selected.try_cast::<GeneratedGridMap>() {
            ggm.bind_mut().place_default();
        }
    }

    fn on_clr_button_press(&mut self) {

        let mut selection : Gd<EditorSelection>;

        {
            let interface = self.base_mut().get_editor_interface();
            if interface == None {
                return;
            }
            let interface = interface.unwrap();

            let oselection = interface.get_selection();
            if oselection == None {
                return;
            }
            selection = oselection.unwrap();
        }

        if selection.get_selected_nodes().len() != 1 {
            return;
        }

        let selected = selection.get_selected_nodes().at(0);

        if let Ok(mut ggm) = selected.try_cast::<GeneratedGridMap>() {
            ggm.clear();
        }
    }

}

#[godot_api]
impl IEditorPlugin for ButtonsPlugin {
    fn enter_tree(&mut self) {
        // Perform typical plugin operations here.
        let mut gen_button = Button::new_alloc();

        gen_button.set_text("Generate");

        gen_button.hide();

        gen_button.signals().pressed().connect_other( self, |this: &mut Self| this.on_gen_button_press() );

        self.base_mut().add_control_to_container( CustomControlContainer::SPATIAL_EDITOR_MENU, &gen_button );

        self.gen_button = Some(gen_button);



        let mut pla_button = Button::new_alloc();

        pla_button.set_text("Place Generated Map");

        pla_button.hide();

        pla_button.signals().pressed().connect_other( self, |this: &mut Self| this.on_pla_button_press() );

        self.base_mut().add_control_to_container( CustomControlContainer::SPATIAL_EDITOR_MENU, &pla_button );

        self.pla_button = Some(pla_button);


        let mut clr_button = Button::new_alloc();

        clr_button.set_text("Clear Map");

        clr_button.hide();

        clr_button.signals().pressed().connect_other( self, |this: &mut Self| this.on_clr_button_press() );

        self.base_mut().add_control_to_container( CustomControlContainer::SPATIAL_EDITOR_MENU, &clr_button );

        self.clr_button = Some(clr_button);


        let selection : Gd<EditorSelection>;

        {
            let interface = self.base_mut().get_editor_interface();
            if interface == None {
                return;
            }
            let interface = interface.unwrap();

            let oselection = interface.get_selection();
            if oselection == None {
                return;
            }
            selection = oselection.unwrap();
        }

        selection.signals().selection_changed().connect_other( self, |this: &mut Self| this.check_selection() );
    }

    fn exit_tree(&mut self) {
        self.gen_button.take();
        self.pla_button.take();
    }
}
