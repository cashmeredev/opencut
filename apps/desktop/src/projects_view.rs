use gpui::{Context, SharedString, WeakEntity, Window, div, prelude::*, rgb};
use scene::ProjectMetadata;
use storage::ProjectStore;

use crate::app::OpenCutApp;
use crate::{projects_root, theme};

pub struct ProjectsView {
    store: ProjectStore,
    projects: Vec<ProjectMetadata>,
    load_error: Option<String>,
    app: WeakEntity<OpenCutApp>,
}

impl ProjectsView {
    pub fn new(app: WeakEntity<OpenCutApp>) -> Self {
        let store = ProjectStore::new(projects_root());
        let (projects, load_error) = match store.list_projects() {
            Ok(projects) => (projects, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        };
        Self {
            store,
            projects,
            load_error,
            app,
        }
    }

    fn reload(&mut self) {
        match self.store.list_projects() {
            Ok(projects) => {
                self.projects = projects;
                self.load_error = None;
            }
            Err(error) => self.load_error = Some(error.to_string()),
        }
    }

    fn create_project(&mut self, cx: &mut Context<Self>) {
        let project = scene::new_project("Untitled Project");
        if self.store.create_project(&project).is_ok() {
            self.open_project(project.metadata.id, project.metadata.name, cx);
        }
    }

    fn open_project(&mut self, id: String, name: String, cx: &mut Context<Self>) {
        let _ = self.app.update(cx, |app, cx| app.open_project(id, name, cx));
    }

    fn delete_project(&mut self, id: &str, cx: &mut Context<Self>) {
        let _ = self.store.delete_project(id);
        self.reload();
        cx.notify();
    }
}

impl Render for ProjectsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = div().flex().flex_col().gap_2().w_full();

        if let Some(error) = &self.load_error {
            list = list.child(
                div()
                    .text_color(rgb(0xef4444))
                    .child(format!("Failed to load projects: {error}")),
            );
        }

        if self.projects.is_empty() && self.load_error.is_none() {
            list = list.child(
                div()
                    .text_color(rgb(theme::TEXT_SECONDARY))
                    .child("No projects yet. Create one to get started."),
            );
        }

        for project in &self.projects {
            let id = project.id.clone();
            let name = project.name.clone();
            let delete_id = project.id.clone();
            let updated = project.updated_at.format("%Y-%m-%d %H:%M").to_string();
            let duration = project.duration.to_seconds_f64();

            list = list.child(
                div()
                    .id(SharedString::from(format!("project-{id}")))
                    .w_full()
                    .px_4()
                    .py_3()
                    .bg(rgb(theme::SURFACE))
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(theme::BORDER))
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(theme::SURFACE_HOVER)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_project(id.clone(), name.clone(), cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().child(project.name.clone()))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme::TEXT_SECONDARY))
                                    .child(format!("{duration:.1}s · updated {updated}")),
                            ),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("delete-{delete_id}")))
                            .px_2()
                            .py_1()
                            .text_sm()
                            .text_color(rgb(theme::TEXT_SECONDARY))
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|style| style.text_color(rgb(0xef4444)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.delete_project(&delete_id, cx);
                                cx.stop_propagation();
                            }))
                            .child("Delete"),
                    ),
            );
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_8()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .child(div().text_xl().child("Projects"))
                    .child(
                        div()
                            .id("new-project")
                            .px_4()
                            .py_2()
                            .bg(rgb(theme::ACCENT))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x2563eb)))
                            .on_click(cx.listener(|this, _, _, cx| this.create_project(cx)))
                            .child("New project"),
                    ),
            )
            .child(div().flex_1().overflow_y_hidden().child(list))
    }
}
