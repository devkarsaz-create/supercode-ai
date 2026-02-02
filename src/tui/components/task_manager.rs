//! Professional Task Management System
//!
//! سیستم مدیریت تسک حرفه‌ای مشابه ClaudeCode/ClaudeTask
//! با قابلیت‌های:
//! - Non-blocking task execution
//! - Task priority queue
//! - Task dependencies
//! - Progress tracking
//! - Task queuing

use crate::tui::state::{Task, TaskStatus, Priority, TaskManager as BaseTaskManager, TaskId};
use crate::agent::sub_agent::SubAgent;
use crate::llm::Llm;
use crate::memory::store::MemoryStore;
use crate::tools::registry::ToolRegistry;
use chrono::{DateTime, Local};
use std::collections::{HashMap, VecDeque, HashSet, BinaryHeap};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// نوع‌های Task Event
#[derive(Debug, Clone)]
pub enum TaskEvent {
    Created(TaskId),
    Started(TaskId),
    StepCompleted(TaskId, String),
    Message(TaskId, String),
    Completed(TaskId),
    Failed(TaskId, String),
    Paused(TaskId),
    Resumed(TaskId),
    Cancelled(TaskId),
    Progress(TaskId, f64),
}

/// یک مرحله از تسک
#[derive(Debug, Clone)]
pub struct TaskStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: StepStatus,
    pub result: Option<String>,
    pub started_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
    pub depends_on: Vec<String>, // ID مراحل وابسته
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Ready,          // آماده اجرا (وابستگی‌ها تکمیل)
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl Default for StepStatus {
    fn default() -> Self {
        StepStatus::Pending
    }
}

/// Task کامل با تمام ویژگی‌ها
#[derive(Debug, Clone)]
pub struct ProfessionalTask {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub steps: Vec<TaskStep>,
    pub current_step: Option<String>,
    pub progress: f64,
    pub agent: Option<TaskAgent>,
    pub context: TaskContext,
    pub dependencies: Vec<TaskId>, // تسک‌های وابسته
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub started_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
    pub parent_id: Option<TaskId>, // برای subtasks
    pub tags: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TaskAgent {
    pub name: String,
    pub role: String,
    pub llm: Arc<dyn Llm>,
    pub tools: ToolRegistry,
    pub memory: MemoryStore,
    pub state: AgentState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Thinking,
    Acting,
    Waiting,
}

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub files: Vec<String>,
    pub messages: Vec<String>,
    pub goals: Vec<String>,
    pub constraints: Vec<String>,
    pub output_format: Option<String>,
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            files: vec![],
            messages: vec![],
            goals: vec![],
            constraints: vec![],
            output_format: None,
        }
    }
}

impl Default for ProfessionalTask {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::new(),
            description: String::new(),
            status: TaskStatus::Pending,
            priority: Priority::Medium,
            steps: vec![],
            current_step: None,
            progress: 0.0,
            agent: None,
            context: TaskContext::default(),
            dependencies: vec![],
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            parent_id: None,
            tags: vec![],
            notes: vec![],
        }
    }
}

impl ProfessionalTask {
    pub fn new(title: &str) -> Self {
        let mut task = Self::default();
        task.title = title.to_string();
        task
    }

    pub fn add_step(&mut self, title: &str, description: &str, depends_on: Vec<String>) -> String {
        let step_id = Uuid::new_v4().to_string();
        self.steps.push(TaskStep {
            id: step_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            status: StepStatus::Pending,
            result: None,
            started_at: None,
            completed_at: None,
            depends_on,
        });
        step_id
    }

    pub fn update_progress(&mut self) {
        if self.steps.is_empty() {
            self.progress = match self.status {
                TaskStatus::Completed => 1.0,
                TaskStatus::InProgress => 0.1,
                _ => 0.0,
            };
        } else {
            let total = self.steps.len() as f64;
            let completed: f64 = self.steps.iter()
                .filter(|s| s.status == StepStatus::Completed)
                .count() as f64;
            self.progress = completed / total;
        }
        self.updated_at = Local::now();
    }

    pub fn can_start(&self, completed_tasks: &HashSet<TaskId>) -> bool {
        // بررسی وابستگی‌ها
        for dep in &self.dependencies {
            if !completed_tasks.contains(dep) {
                return false;
            }
        }
        
        // بررسی آمادگی مراحل
        for step in &self.steps {
            if step.status == StepStatus::Pending {
                // بررسی وابستگی‌های مرحله
                let mut all_deps_done = true;
                for dep_id in &step.depends_on {
                    let dep_step = self.steps.iter().find(|s| s.id == dep_id);
                    if let Some(s) = dep_step {
                        if s.status != StepStatus::Completed {
                            all_deps_done = false;
                            break;
                        }
                    }
                }
                if all_deps_done {
                    return true; // حداقل یک مرحله آماده است
                }
            }
        }
        
        false
    }

    pub fn get_next_ready_step(&mut self) -> Option<&mut TaskStep> {
        for step in &mut self.steps {
            if step.status == StepStatus::Pending {
                let mut all_deps_done = true;
                for dep_id in &step.depends_on {
                    let dep_step = self.steps.iter().find(|s| s.id == dep_id);
                    if let Some(s) = dep_step {
                        if s.status != StepStatus::Completed {
                            all_deps_done = false;
                            break;
                        }
                    }
                }
                if all_deps_done {
                    step.status = StepStatus::InProgress;
                    step.started_at = Some(Local::now());
                    self.current_step = Some(step.id.clone());
                    self.status = TaskStatus::InProgress;
                    return Some(step);
                }
            }
        }
        None
    }
}

/// Task Queue با Priority
#[derive(Clone)]
pub struct TaskQueue {
    queue: Arc<RwLock<BinaryHeap<TaskWrapper>>>,
    completed: Arc<RwLock<HashSet<TaskId>>>,
    event_sender: mpsc::UnboundedSender<TaskEvent>,
}

#[derive(Clone, Debug)]
struct TaskWrapper {
    task: Arc<RwLock<ProfessionalTask>>,
    priority: i32, // اولویت معکوس (عدد کوچکتر = اولویت بالاتر)
}

impl Eq for TaskWrapper {}
impl PartialEq for TaskWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Ord for TaskWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // برعکس کردن برای min-heap
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for TaskWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TaskQueue {
    pub fn new(event_sender: mpsc::UnboundedSender<TaskEvent>) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            completed: Arc::new(RwLock::new(HashSet::new())),
            event_sender,
        }
    }

    pub async fn enqueue(&self, task: ProfessionalTask, priority: i32) {
        let task_arc = Arc::new(RwLock::new(task));
        self.queue.write().push(TaskWrapper {
            task: task_arc,
            priority,
        });
        
        // اطلاع‌رسانی
        let task_id = task_arc.read().id.clone();
        let _ = self.event_sender.send(TaskEvent::Created(task_id));
    }

    pub async fn dequeue(&self, completed_tasks: &HashSet<TaskId>) -> Option<Arc<RwLock<ProfessionalTask>>> {
        let mut queue = self.queue.write();
        
        while let Some(wrapper) = queue.pop() {
            let task = wrapper.task.read().await;
            if task.can_start(completed_tasks) {
                drop(task);
                return Some(wrapper.task);
            }
            // اگر نمی‌تواند شروع شود، دوباره اضافه کن
            queue.push(wrapper);
            return None;
        }
        
        None
    }

    pub async fn complete(&self, task_id: &TaskId) {
        self.completed.write().insert(task_id.clone());
        let _ = self.event_sender.send(TaskEvent::Completed(task_id.clone()));
    }

    pub async fn fail(&self, task_id: &TaskId, error: &str) {
        let _ = self.event_sender.send(TaskEvent::Failed(task_id.clone(), error.to_string()));
    }

    pub fn get_completed_count(&self) -> usize {
        self.completed.read().len()
    }

    pub fn get_queue_size(&self) -> usize {
        self.queue.read().len()
    }
}

/// مدیر تسک پیشرفته
#[derive(Clone)]
pub struct ProfessionalTaskManager {
    pub tasks: Arc<RwLock<HashMap<TaskId, Arc<RwLock<ProfessionalTask>>>>>,
    pub queue: TaskQueue,
    pub running_task: Arc<RwLock<Option<Arc<RwLock<ProfessionalTask>>>>>,
    pub task_history: Arc<RwLock<Vec<TaskId>>>,
    pub event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<TaskEvent>>>,
    pub base_task_manager: Arc<BaseTaskManager>,
}

impl ProfessionalTaskManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            queue: TaskQueue::new(sender.clone()),
            running_task: Arc::new(RwLock::new(None)),
            task_history: Arc::new(RwLock::new(Vec::new())),
            event_receiver: Arc::new(RwLock::new(receiver)),
            base_task_manager: Arc::new(BaseTaskManager::new()),
        }
    }

    /// ایجاد تسک جدید
    pub async fn create_task(&self, title: &str, description: &str, priority: Priority) -> TaskId {
        let mut task = ProfessionalTask::new(title);
        task.description = description.to_string();
        task.priority = priority;
        
        let task_id = task.id.clone();
        let task_arc = Arc::new(RwLock::new(task));
        
        self.tasks.write().insert(task_id.clone(), task_arc.clone());
        
        // اضافه کردن به صف با اولویت
        let priority_num = match priority {
            Priority::Critical => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        };
        
        self.queue.enqueue(task_arc.read().await.clone(), priority_num).await;
        
        task_id
    }

    /// افزودن مرحله به تسک
    pub async fn add_step(&self, task_id: &TaskId, title: &str, description: &str, depends_on: Vec<String>) -> Option<String> {
        let tasks = self.tasks.read();
        if let Some(task_arc) = tasks.get(task_id) {
            let mut task = task_arc.write().await;
            let step_id = task.add_step(title, description, depends_on);
            task.update_progress();
            return Some(step_id);
        }
        None
    }

    /// شروع تسک بعدی
    pub async fn start_next_task(&self) -> Option<Arc<RwLock<ProfessionalTask>>> {
        let completed = self.get_completed_set().await;
        
        if let Some(task_arc) = self.queue.dequeue(&completed).await {
            let mut task = task_arc.write().await;
            task.status = TaskStatus::InProgress;
            task.started_at = Some(Local::now());
            task.update_progress();
            
            drop(task);
            
            *self.running_task.write() = Some(task_arc.clone());
            
            // اطلاع‌رسانی
            let task_id = task_arc.read().id.clone();
            let _ = self.queue.event_sender.send(TaskEvent::Started(task_id));
            
            // اضافه کردن به history
            self.task_history.write().push(task_id.clone());
            
            return Some(task_arc);
        }
        
        None
    }

    /// تکمیل مرحله
    pub async fn complete_step(&self, task_id: &TaskId, step_id: &str, result: &str) -> bool {
        let tasks = self.tasks.read();
        if let Some(task_arc) = tasks.get(task_id) {
            let mut task = task_arc.write().await;
            
            for step in &mut task.steps {
                if step.id == step_id {
                    step.status = StepStatus::Completed;
                    step.result = Some(result.to_string());
                    step.completed_at = Some(Local::now());
                    
                    task.update_progress();
                    
                    // اطلاع‌رسانی
                    let _ = self.queue.event_sender.send(TaskEvent::StepCompleted(
                        task_id.clone(), 
                        step.title.clone()
                    ));
                    
                    // بررسی تسک کامل شده
                    if task.steps.iter().all(|s| s.status == StepStatus::Completed) {
                        task.status = TaskStatus::Completed;
                        task.completed_at = Some(Local::now());
                        task.progress = 1.0;
                        self.queue.complete(task_id).await;
                    } else if let Some(next_step) = task.get_next_ready_step() {
                        let _ = self.queue.event_sender.send(TaskEvent::Progress(
                            task_id.clone(),
                            task.progress
                        ));
                    }
                    
                    return true;
                }
            }
        }
        false
    }

    /// دریافت تمام تسک‌ها
    pub async fn get_all_tasks(&self) -> Vec<Arc<RwLock<ProfessionalTask>>> {
        self.tasks.read().values().cloned().collect()
    }

    /// دریافت تسک با ID
    pub async fn get_task(&self, task_id: &TaskId) -> Option<Arc<RwLock<ProfessionalTask>>> {
        self.tasks.read().get(task_id).cloned()
    }

    /// دریافت تسک در حال اجرا
    pub async fn get_running_task(&self) -> Option<Arc<RwLock<ProfessionalTask>>> {
        self.running_task.read().clone()
    }

    /// دریافت مجموعه تسک‌های تکمیل‌شده
    async fn get_completed_set(&self) -> HashSet<TaskId> {
        let tasks = self.tasks.read();
        tasks.values()
            .filter(|t| t.read().await.status == TaskStatus::Completed)
            .map(|t| t.read().await.id.clone())
            .collect()
    }

    /// دریافت آمار تسک‌ها
    pub async fn get_stats(&self) -> TaskStats {
        let tasks = self.tasks.read();
        
        let mut stats = TaskStats::default();
        
        for task_arc in tasks.values() {
            let task = task_arc.read().await;
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::InProgress => stats.in_progress += 1,
                TaskStatus::Paused => stats.paused += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Failed => stats.failed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
            }
            
            stats.total_steps += task.steps.len();
            stats.completed_steps += task.steps.iter()
                .filter(|s| s.status == StepStatus::Completed)
                .count();
        }
        
        stats.queue_size = self.queue.get_queue_size();
        stats.total_tasks = tasks.len();
        
        stats
    }

    /// دریافت تسک‌های مرتب‌شده
    pub async fn get_tasks_by_priority(&self) -> Vec<Arc<RwLock<ProfessionalTask>>> {
        let tasks = self.tasks.read();
        let mut all_tasks: Vec<_> = tasks.values().collect();
        all_tasks.sort_by(|a, b| {
            let a_priority = a.read().await.priority.clone() as u8;
            let b_priority = b.read().await.priority.clone() as u8;
            b_priority.cmp(&a_priority) // اولویت بالاتر اول
        });
        all_tasks
    }
}

/// آمار تسک‌ها
#[derive(Debug, Default, Clone)]
pub struct TaskStats {
    pub total_tasks: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub paused: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub queue_size: usize,
}

impl TaskStats {
    pub fn completion_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.completed as f64 / self.total_tasks as f64
        }
    }

    pub fn step_completion_rate(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            self.completed_steps as f64 / self.total_steps as f64
        }
    }
}

/// رندر کردن Task Panel
pub fn render_task_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &ProfessionalTaskManager,
    area: Rect,
    theme: &crate::config::ThemeName,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let stats = rt.block_on(manager.get_stats());
    
    let (bg_color, fg_color, highlight_color) = match theme {
        crate::config::ThemeName::DarkPlus => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
        _ => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
    };

    // آمار بالا
    let stats_content = format!(
        "📊 TASK STATISTICS\n\n\
        📋 Total: {} | 🔄 In Progress: {} | ⏳ Pending: {}\n\
        ✅ Completed: {} | ❌ Failed: {} | 📦 Queue: {}\n\n\
        📈 Progress: {:.1}% overall\n\
        📝 Steps: {}/{} ({:.1}%)",
        stats.total_tasks,
        stats.in_progress,
        stats.pending,
        stats.completed,
        stats.failed,
        stats.queue_size,
        stats.completion_rate() * 100.0,
        stats.completed_steps,
        stats.total_steps,
        stats.step_completion_rate() * 100.0
    );

    let panel = Paragraph::new(stats_content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().title("📋 Task Manager").borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// رندر کردن Task Queue
pub fn render_task_queue<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &TaskQueue,
    area: Rect,
    theme: &crate::config::ThemeName,
) {
    let (bg_color, fg_color, _) = match theme {
        crate::config::ThemeName::DarkPlus => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
        _ => (Color::Rgb(30, 30, 30), Color::White, Color::Cyan),
    };

    let queue_size = manager.get_queue_size();
    let completed = manager.get_completed_count();

    let content = format!(
        "📦 TASK QUEUE\n\n\
        ⏳ Waiting: {}\n\
        ✅ Completed: {}\n\n\
        Queue is processing tasks in priority order.",
        queue_size,
        completed
    );

    let panel = Paragraph::new(content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().title("📦 Queue").borders(Borders::NONE));

    frame.render_widget(panel, area);
}
