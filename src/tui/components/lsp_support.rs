//! LSP Support for Super-Agent TUI
//!
//! پشتیبانی از Language Server Protocol:
//! - Auto-completion
//! - Diagnostics
//! - Go to definition
//! - Find references
//! - Hover information
//! - Code actions

use crate::tools::registry::ToolRegistry;
use crate::types::ToolResult;
use async_trait::async_trait;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::process::Command;
use tokio::sync::mpsc;
use uuid::Uuid;

/// یک LSP Server
#[derive(Debug, Clone)]
pub struct LspServer {
    pub id: String,
    pub name: String,
    pub language_id: String,
    pub command: Vec<String>,
    pub workspace_root: PathBuf,
    pub capabilities: LspCapabilities,
    pub status: LspServerStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LspServerStatus {
    NotStarted,
    Starting,
    Running,
    Stopping,
    Failed(String),
}

/// قابلیت‌های LSP Server
#[derive(Debug, Clone, Default)]
pub struct LspCapabilities {
    pub completion_provider: bool,
    pub hover_provider: bool,
    pub definition_provider: bool,
    pub references_provider: bool,
    pub document_formatting_provider: bool,
    pub code_action_provider: bool,
    pub diagnostic_provider: bool,
    pub rename_provider: bool,
}

/// درخواست LSP
#[derive(Debug, Clone)]
pub enum LspRequest {
    /// درخواست تکمیل خودکار
    Complete {
        file_path: PathBuf,
        line: u32,
        character: u32,
    },
    /// درخواست hover
    Hover {
        file_path: PathBuf,
        line: u32,
        character: u32,
    },
    /// رفتن به تعریف
    GotoDefinition {
        file_path: PathBuf,
        line: u32,
        character: u32,
    },
    /// یافتن مراجع
    FindReferences {
        file_path: PathBuf,
        line: u32,
        character: u32,
    },
    /// فرمت کردن سند
    FormatDocument {
        file_path: PathBuf,
    },
    /// کد اکشن‌ها
    CodeActions {
        file_path: PathBuf,
        line: u32,
        character: u32,
        diagnostics: Vec<Diagnostic>,
    },
    /// تغییر نام
    Rename {
        file_path: PathBuf,
        line: u32,
        character: u32,
        new_name: String,
    },
}

/// پاسخ LSP
#[derive(Debug, Clone)]
pub enum LspResponse {
    /// نتایج تکمیل خودکار
    Completion(CompletionResult),
    /// اطلاعات hover
    Hover(HoverResult),
    /// موقعیت تعریف
    Definition(DefinitionResult),
    /// لیست مراجع
    References(ReferencesResult),
    /// فرمت سند
    DocumentFormat(DocumentFormatResult),
    /// کد اکشن‌ها
    CodeActions(CodeActionsResult),
    /// نتیجه تغییر نام
    Rename(RenameResult),
    /// خطا
    Error(String),
}

/// نتیجه تکمیل خودکار
#[derive(Debug, Clone, Default)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub is_incomplete: bool,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub filter_text: Option<String>,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
    Default,
}

/// نتیجه hover
#[derive(Debug, Clone, Default)]
pub struct HoverResult {
    pub contents: Vec<HoverContent>,
    pub range: Option<Range>,
}

#[derive(Debug, Clone)]
pub struct HoverContent {
    pub language: Option<String>,
    pub value: String,
}

/// موقعیت تعریف
#[derive(Debug, Clone, Default)]
pub struct DefinitionResult {
    pub locations: Vec<Location>,
}

/// لیست مراجع
#[derive(Debug, Clone, Default)]
pub struct ReferencesResult {
    pub references: Vec<Location>,
}

/// فرمت سند
#[derive(Debug, Clone, Default)]
pub struct DocumentFormatResult {
    pub edits: Vec<TextEdit>,
}

/// کد اکشن‌ها
#[derive(Debug, Clone, Default)]
pub struct CodeActionsResult {
    pub actions: Vec<CodeAction>,
}

/// نتیجه تغییر نام
#[derive(Debug, Clone, Default)]
pub struct RenameResult {
    pub changes: Vec<FileEdit>,
}

/// موقعیت در فایل
#[derive(Debug, Clone)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Default)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// ویرایش متن
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// ویرایش فایل
#[derive(Debug, Clone)]
pub struct FileEdit {
    pub uri: String,
    pub edits: Vec<TextEdit>,
}

/// اکشن کد
#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
    pub edit: Option<TextEdit>,
    pub command: Option<CommandInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeActionKind {
    QuickFix,
    Refactor,
    RefactorExtract,
    RefactorInline,
    RefactorRewrite,
    Source,
    SourceOrganizeImports,
    Default,
}

/// دیاگنوستیک
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
    Default,
}

/// مدیر LSP
#[derive(Clone)]
pub struct LspManager {
    pub servers: Arc<RwLock<HashMap<String, LspServer>>>,
    pub open_files: Arc<RwLock<HashSet<PathBuf>>>,
    pub diagnostics: Arc<RwLock<HashMap<PathBuf, Vec<Diagnostic>>>>,
    pub request_sender: mpsc::UnboundedSender<(LspRequest, String)>,
    pub response_receiver: Arc<RwLock<mpsc::UnboundedReceiver<(String, LspResponse)>>>,
}

impl Default for LspManager {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            open_files: Arc::new(RwLock::new(HashSet::new())),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            request_sender: sender,
            response_receiver: Arc::new(RwLock::new(receiver)),
        }
    }
}

impl LspManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// ثبت LSP Server جدید
    pub async fn register_server(&self, server: LspServer) {
        let mut servers = self.servers.write();
        servers.insert(server.id.clone(), server);
    }

    /// شروع LSP Server
    pub async fn start_server(&self, server_id: &str) -> Result<(), String> {
        let servers = self.servers.read();
        if let Some(server) = servers.get(server_id) {
            let mut s = server.clone();
            s.status = LspServerStatus::Starting;
            
            // TODO: پیاده‌سازی واقعی با tokio::process
            s.status = LspServerStatus::Running;
            
            let mut servers = self.servers.write();
            if let Some(existing) = servers.get_mut(server_id) {
                existing.status = s.status;
            }
            Ok(())
        } else {
            Err(format!("Server {} not found", server_id))
        }
    }

    /// باز کردن فایل
    pub async fn open_file(&self, file_path: &PathBuf) {
        let mut open_files = self.open_files.write();
        open_files.insert(file_path.clone());
    }

    /// بستن فایل
    pub async fn close_file(&self, file_path: &PathBuf) {
        let mut open_files = self.open_files.write();
        open_files.remove(file_path);
        
        let mut diagnostics = self.diagnostics.write();
        diagnostics.remove(file_path);
    }

    /// درخواست تکمیل خودکار
    pub async fn request_completion(&self, file_path: PathBuf, line: u32, character: u32) -> CompletionResult {
        CompletionResult::default()
    }

    /// درخواست hover
    pub async fn request_hover(&self, file_path: PathBuf, line: u32, character: u32) -> HoverResult {
        HoverResult::default()
    }

    /// درخواست رفتن به تعریف
    pub async fn request_definition(&self, file_path: PathBuf, line: u32, character: u32) -> DefinitionResult {
        DefinitionResult::default()
    }

    /// دریافت دیاگنوستیک‌های فایل
    pub async fn get_diagnostics(&self, file_path: &PathBuf) -> Vec<Diagnostic> {
        let diagnostics = self.diagnostics.read();
        diagnostics.get(file_path).cloned().unwrap_or_default()
    }

    /// دریافت تمام دیاگنوستیک‌ها
    pub async fn get_all_diagnostics(&self) -> Vec<(PathBuf, Vec<Diagnostic>)> {
        let diagnostics = self.diagnostics.read();
        diagnostics.iter()
            .map(|(path, diags)| (path.clone(), diags.clone()))
            .collect()
    }
}

/// ابزار LSP برای agent
#[async_trait]
pub trait LspTool {
    async fn complete(&self, file: &str, line: u32, character: u32) -> ToolResult<CompletionResult>;
    async fn hover(&self, file: &str, line: u32, character: u32) -> ToolResult<HoverResult>;
    async fn goto_definition(&self, file: &str, line: u32, character: u32) -> ToolResult<DefinitionResult>;
    async fn find_references(&self, file: &str, line: u32, character: u32) -> ToolResult<ReferencesResult>;
    async fn format_document(&self, file: &str) -> ToolResult<DocumentFormatResult>;
    async fn code_actions(&self, file: &str, line: u32, character: u32) -> ToolResult<CodeActionsResult>;
    async fn rename(&self, file: &str, line: u32, character: u32, new_name: &str) -> ToolResult<RenameResult>;
}

/// رندر LSP Diagnostics Panel
pub fn render_lsp_panel<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &LspManager,
    area: Rect,
    theme: &crate::config::ThemeName,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let diagnostics = rt.block_on(manager.get_all_diagnostics());
    
    let (bg_color, fg_color, error_color, warning_color) = match theme {
        crate::config::ThemeName::DarkPlus => (
            Color::Rgb(30, 30, 30),
            Color::White,
            Color::Red,
            Color::Yellow,
        ),
        _ => (
            Color::Rgb(30, 30, 30),
            Color::White,
            Color::Red,
            Color::Yellow,
        ),
    };

    let mut content = format!(
        "🔍 LSP DIAGNOSTICS\n\n\
        📁 Open Files: {}\n\
        📊 Total Diagnostics: {}\n\n\
        🔴 Errors: {}\n\
        🟡 Warnings: {}\n\n\
        📋 DIAGNOSTICS LIST\n",
        manager.open_files.read().len(),
        diagnostics.iter().map(|(_, d)| d.len()).sum::<usize>(),
        diagnostics.iter()
            .flat_map(|(_, d)| d.iter())
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count(),
        diagnostics.iter()
            .flat_map(|(_, d)| d.iter())
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    );

    for (path, diags) in diagnostics {
        for diag in diags {
            let (icon, color) = match diag.severity {
                DiagnosticSeverity::Error => ("🔴", error_color),
                DiagnosticSeverity::Warning => ("🟡", warning_color),
                _ => ("🔵", fg_color),
            };
            content.push_str(&format!(
                "{} {}:{}\n   {}\n",
                icon,
                path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                diag.range.start.line + 1,
                diag.message
            ));
        }
    }

    let panel = Paragraph::new(content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(Block::default().title("🔍 LSP Diagnostics").borders(Borders::NONE));

    frame.render_widget(panel, area);
}

/// LSP Status Bar Item
pub fn render_lsp_status<B: ratatui::backend::Backend>(
    frame: &mut Frame<B>,
    manager: &LspManager,
    area: Rect,
) {
    let servers = manager.servers.read();
    let running = servers.values()
        .filter(|s| s.status == LspServerStatus::Running)
        .count();
    let total = servers.len();

    let content = format!(" LSP: {}/{} servers running ", running, total);

    let widget = Paragraph::new(content)
        .style(Style::default().bg(Color::Rgb(40, 40, 40)).fg(Color::Cyan))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(widget, area);
}
