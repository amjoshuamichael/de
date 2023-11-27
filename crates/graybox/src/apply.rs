use bevy::{prelude::*, reflect::*, ecs::component::Components};

use crate::*;

#[derive(Debug)]
pub enum PathKind {
    Field(String), 
}

pub fn apply_modifications<T: Reflect + Component>(
    submenus: Query<(&InspectorSubmenu, &Parent)>,
    inspectors: Query<&Inspector>,
    inspector_sections: Query<(&InspectorSection, &Parent)>,
    mut modifications: ResMut<ModificationEvents>,
    mut datas: Query<&mut T>,
    components: &Components,
) {
    let component_id = components.component_id::<T>().unwrap();

    for m in (0..modifications.0.len()).rev() {
        let modification = &modifications.0[m];
        if modification.component_id != component_id { continue }

        let (path_sequence, section_root): (Vec<&PathKind>, Entity) = {
            let mut sequence = Vec::<&PathKind>::new();
            let mut submenu_entity = modification.editor_in_submenu;

            while let Ok((submenu, parent)) = submenus.get(submenu_entity) {
                sequence.push(&submenu.path_kind);
                submenu_entity = **parent;
            }

            (sequence, submenu_entity)
        };

        let inspector_section = inspector_sections.get(section_root).unwrap();
        let inspector = inspectors.get(**inspector_section.1).unwrap();
        let mut obj: &mut dyn Reflect = &mut *datas.get_mut(inspector.on).unwrap();

        for path_kind in path_sequence.into_iter().rev() {
            match obj.reflect_mut() {
                ReflectMut::Struct(reflect_struct) => {
                    let PathKind::Field(str) = path_kind else { unreachable!() };
                    obj = reflect_struct.field_mut(str).unwrap();
                },
                ReflectMut::TupleStruct(_) => todo!(),
                ReflectMut::Tuple(_) => todo!(),
                ReflectMut::List(_) => todo!(),
                ReflectMut::Array(_) => todo!(),
                ReflectMut::Map(_) => todo!(),
                ReflectMut::Enum(_) => todo!(),
                ReflectMut::Value(_) => todo!(),
            }
        }

        let modification = modifications.0.remove(m);
        obj.set(modification.set).expect("failure setting value");
    }
}
