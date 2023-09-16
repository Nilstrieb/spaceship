use std::any::TypeId;

use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalForce;
use indexmap::IndexMap;

#[derive(Component, Default)]
pub struct ExternalForceSet {
    forces: IndexMap<TypeId, ExternalForce, ahash::RandomState>,
}

impl ExternalForceSet {
    pub fn get<T: 'static>(&self) -> ExternalForce {
        self.forces
            .get(&TypeId::of::<T>())
            .copied()
            .unwrap_or_default()
    }

    pub fn set<T: 'static>(&mut self, force: ExternalForce) {
        self.forces.insert(TypeId::of::<T>(), force);
    }

    fn combine(&self) -> ExternalForce {
        self.forces
            .values()
            .fold(ExternalForce::default(), |f1, &f2| f1 + f2)
    }
}

pub fn update_external_forces(mut query: Query<(&mut ExternalForce, &ExternalForceSet)>) {
    for (mut force, forces) in &mut query {
        *force = forces.combine();
    }
}
