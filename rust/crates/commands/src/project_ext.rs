use scene::{Project, Scene};

pub fn active_scene(project: &Project) -> Option<&Scene> {
    project
        .scenes
        .iter()
        .find(|scene| scene.id == project.current_scene_id)
        .or_else(|| project.scenes.iter().find(|scene| scene.is_main))
        .or_else(|| project.scenes.first())
}

pub fn active_scene_mut(project: &mut Project) -> Option<&mut Scene> {
    let index = project
        .scenes
        .iter()
        .position(|scene| scene.id == project.current_scene_id)
        .or_else(|| project.scenes.iter().position(|scene| scene.is_main))
        .or(if project.scenes.is_empty() { None } else { Some(0) })?;
    project.scenes.get_mut(index)
}
