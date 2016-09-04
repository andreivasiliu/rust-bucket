/*
 * The world state contains information about the objects, bodies, and walls of
 * the world, and any active events happening inside it. This state is identical
 * for all players; information about what each player controls is stored
 * outside of the world state, in each player's GameState.
 */

extern crate vecmath;

use std::collections::BTreeMap;
use vecmath::Vector2;

pub struct World {
    // TODO: Remove pub
    pub objects: BTreeMap<ObjectName, Object>,
}

impl World {
    pub fn new() -> World {
        return World {
            objects: BTreeMap::new(),
        }
    }

    pub fn spawn_object(&mut self) -> ObjectName {
        let object = Object {
            position: [0, 0],
            inertia: [0, 0],
        };
        let name = ObjectName { id: 0 };
        self.objects.insert(name, object);
        return name;
    }

    pub fn set_object_inertia(&mut self, name: &ObjectName, inertia: [i32; 2]) {
        if let Some(object) = self.objects.get_mut(name) {
            object.set_inertia(inertia);
        }
    }

    // pub fn get_object<'a>(&'a mut self, name: ObjectName) -> Option<&mut Object> {
    //     return self.objects.get_mut(&name);
    // }

    pub fn update(&mut self) {
        for (_, object) in self.objects.iter_mut() {
            object.update();
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone, Hash)]
pub struct ObjectName {
    id: usize,
}



pub struct Object {
    position: Vector2<i32>,
    inertia: Vector2<i32>,
}

impl Object {
    pub fn set_inertia(&mut self, inertia: Vector2<i32>) {
        self.inertia = inertia;
    }

    pub fn get_position(&self) -> Vector2<i32> {
        return self.position;
    }

    pub fn update(&mut self) {
        self.position = vecmath::vec2_add(self.position, self.inertia);
    }
}
