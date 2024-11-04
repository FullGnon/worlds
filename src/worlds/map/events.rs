use bevy::prelude::Event;

#[derive(Event)]
pub(crate) struct DrawMapEvent;

#[derive(Event)]
pub(crate) struct GenerateMapEvent;
