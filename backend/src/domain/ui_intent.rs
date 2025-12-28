use serde::{Deserialize, Serialize};

/// Internal DSL for representing screen generation intent.
/// This is the normalized representation that the prompt compiler uses.
/// LLM receives structured intent, not raw input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiIntent {
    /// Screen name (e.g., "member_list", "order_detail")
    pub screen_name: String,

    /// Type of screen to generate
    pub screen_type: ScreenType,

    /// Datasets (data bindings)
    pub datasets: Vec<DatasetIntent>,

    /// Grid components
    pub grids: Vec<GridIntent>,

    /// Available actions/buttons
    pub actions: Vec<ActionIntent>,

    /// Additional notes or requirements
    pub notes: Option<String>,
}

impl UiIntent {
    pub fn new(screen_name: impl Into<String>, screen_type: ScreenType) -> Self {
        Self {
            screen_name: screen_name.into(),
            screen_type,
            datasets: Vec::new(),
            grids: Vec::new(),
            actions: Vec::new(),
            notes: None,
        }
    }

    pub fn with_dataset(mut self, dataset: DatasetIntent) -> Self {
        self.datasets.push(dataset);
        self
    }

    pub fn with_grid(mut self, grid: GridIntent) -> Self {
        self.grids.push(grid);
        self
    }

    pub fn with_action(mut self, action: ActionIntent) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Screen type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenType {
    /// List screen with search and grid
    List,
    /// Detail/form screen
    Detail,
    /// Popup dialog
    Popup,
    /// List with detail popup
    ListWithPopup,
}

impl ScreenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScreenType::List => "list",
            ScreenType::Detail => "detail",
            ScreenType::Popup => "popup",
            ScreenType::ListWithPopup => "list_with_popup",
        }
    }
}

impl std::fmt::Display for ScreenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Dataset intent - represents a data binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetIntent {
    /// Dataset ID (e.g., "ds_member")
    pub id: String,

    /// Source table name
    pub table_name: Option<String>,

    /// Columns in the dataset
    pub columns: Vec<ColumnIntent>,
}

impl DatasetIntent {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            table_name: None,
            columns: Vec::new(),
        }
    }

    pub fn with_table(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = Some(table_name.into());
        self
    }

    pub fn with_column(mut self, column: ColumnIntent) -> Self {
        self.columns.push(column);
        self
    }

    pub fn with_columns(mut self, columns: Vec<ColumnIntent>) -> Self {
        self.columns = columns;
        self
    }
}

/// Column intent - represents a single column/field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnIntent {
    /// Column name (DB column name)
    pub name: String,

    /// Display label (Korean or localized)
    pub label: String,

    /// UI control type
    pub ui_type: UiType,

    /// Data type hint
    pub data_type: DataType,

    /// Is this field required?
    pub required: bool,

    /// Is this field readonly?
    pub readonly: bool,

    /// Is this a primary key?
    pub is_pk: bool,

    /// Maximum length (for strings)
    pub max_length: Option<u32>,

    /// Additional validation rules
    pub validation: Option<String>,
}

impl ColumnIntent {
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            ui_type: UiType::Input,
            data_type: DataType::String,
            required: false,
            readonly: false,
            is_pk: false,
            max_length: None,
            validation: None,
        }
    }

    pub fn with_ui_type(mut self, ui_type: UiType) -> Self {
        self.ui_type = ui_type;
        self
    }

    pub fn with_data_type(mut self, data_type: DataType) -> Self {
        self.data_type = data_type;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.is_pk = true;
        self.readonly = true;
        self.ui_type = UiType::Hidden;
        self
    }

    pub fn with_max_length(mut self, len: u32) -> Self {
        self.max_length = Some(len);
        self
    }
}

/// UI control type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiType {
    /// Text input field
    Input,
    /// Multi-line text area
    TextArea,
    /// Date picker
    DatePicker,
    /// DateTime picker
    DateTimePicker,
    /// Checkbox
    Checkbox,
    /// Dropdown/Combo box
    Combo,
    /// Radio button group
    Radio,
    /// Hidden field
    Hidden,
    /// Numeric input
    Number,
    /// File upload
    File,
}

impl UiType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UiType::Input => "input",
            UiType::TextArea => "textarea",
            UiType::DatePicker => "datepicker",
            UiType::DateTimePicker => "datetimepicker",
            UiType::Checkbox => "checkbox",
            UiType::Combo => "combo",
            UiType::Radio => "radio",
            UiType::Hidden => "hidden",
            UiType::Number => "number",
            UiType::File => "file",
        }
    }
}

/// Data type hint for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Integer,
    Decimal,
    Boolean,
    Date,
    DateTime,
    Text,
    Binary,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::String => "string",
            DataType::Integer => "integer",
            DataType::Decimal => "decimal",
            DataType::Boolean => "boolean",
            DataType::Date => "date",
            DataType::DateTime => "datetime",
            DataType::Text => "text",
            DataType::Binary => "binary",
        }
    }
}

/// Grid intent - represents a grid/table component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridIntent {
    /// Grid ID
    pub id: String,

    /// Bound dataset ID
    pub dataset_id: String,

    /// Visible columns (subset of dataset columns)
    pub columns: Vec<GridColumnIntent>,

    /// Enable row selection
    pub selectable: bool,

    /// Enable editing
    pub editable: bool,

    /// Enable pagination
    pub paginated: bool,

    /// Rows per page (if paginated)
    pub page_size: Option<u32>,
}

impl GridIntent {
    pub fn new(id: impl Into<String>, dataset_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            dataset_id: dataset_id.into(),
            columns: Vec::new(),
            selectable: true,
            editable: false,
            paginated: true,
            page_size: Some(20),
        }
    }

    pub fn with_column(mut self, column: GridColumnIntent) -> Self {
        self.columns.push(column);
        self
    }

    pub fn with_columns(mut self, columns: Vec<GridColumnIntent>) -> Self {
        self.columns = columns;
        self
    }

    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }

    pub fn not_paginated(mut self) -> Self {
        self.paginated = false;
        self.page_size = None;
        self
    }
}

/// Grid column display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridColumnIntent {
    /// Column name (matches dataset column)
    pub name: String,

    /// Header label
    pub header: String,

    /// Column width (pixels or percentage)
    pub width: Option<String>,

    /// Text alignment
    pub align: Alignment,

    /// Is column sortable?
    pub sortable: bool,

    /// Is column filterable?
    pub filterable: bool,
}

impl GridColumnIntent {
    pub fn new(name: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            header: header.into(),
            width: None,
            align: Alignment::Left,
            sortable: true,
            filterable: false,
        }
    }

    pub fn with_width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn with_align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

    pub fn filterable(mut self) -> Self {
        self.filterable = true;
        self
    }
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// Action/button intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionIntent {
    /// Action ID
    pub id: String,

    /// Button label
    pub label: String,

    /// Action type
    pub action_type: ActionType,

    /// JavaScript function name
    pub function_name: String,

    /// Position/group
    pub position: ActionPosition,
}

impl ActionIntent {
    pub fn new(id: impl Into<String>, label: impl Into<String>, action_type: ActionType) -> Self {
        let id_str = id.into();
        let function_name = format!("fn_{}", id_str);
        Self {
            id: id_str,
            label: label.into(),
            action_type,
            function_name,
            position: ActionPosition::Top,
        }
    }

    pub fn with_function(mut self, function_name: impl Into<String>) -> Self {
        self.function_name = function_name.into();
        self
    }

    pub fn at_bottom(mut self) -> Self {
        self.position = ActionPosition::Bottom;
        self
    }
}

/// Action type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Search/query data
    Search,
    /// Save/submit data
    Save,
    /// Delete data
    Delete,
    /// Add new row/record
    Add,
    /// Open popup
    OpenPopup,
    /// Close popup
    ClosePopup,
    /// Export data
    Export,
    /// Print
    Print,
    /// Custom action
    Custom,
}

/// Action button position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPosition {
    Top,
    Bottom,
    Both,
}

/// Default actions for different screen types
pub fn default_actions_for_screen_type(screen_type: ScreenType) -> Vec<ActionIntent> {
    match screen_type {
        ScreenType::List => vec![
            ActionIntent::new("search", "조회", ActionType::Search),
            ActionIntent::new("add", "신규", ActionType::Add),
            ActionIntent::new("delete", "삭제", ActionType::Delete),
        ],
        ScreenType::Detail => vec![
            ActionIntent::new("save", "저장", ActionType::Save),
            ActionIntent::new("delete", "삭제", ActionType::Delete),
        ],
        ScreenType::Popup => vec![
            ActionIntent::new("save", "저장", ActionType::Save),
            ActionIntent::new("close", "닫기", ActionType::ClosePopup),
        ],
        ScreenType::ListWithPopup => vec![
            ActionIntent::new("search", "조회", ActionType::Search),
            ActionIntent::new("add", "신규", ActionType::Add),
            ActionIntent::new("delete", "삭제", ActionType::Delete),
        ],
    }
}
