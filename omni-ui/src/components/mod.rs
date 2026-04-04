mod agent_rail;
mod chat;
mod kanban;
mod panels;
mod sidebar;
mod tabs;
mod ui;

pub use agent_rail::AgentRail;
pub use chat::ChatContainer;
pub use kanban::KanbanView;
pub use panels::{BackgroundTasksSection, FilesSection, TasksSection};
pub use sidebar::ThreadSidebar;
pub use tabs::FileViewer;
pub use ui::{Button, ButtonVariant, Dialog, Input};
