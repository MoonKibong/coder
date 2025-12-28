# xFrame5 Knowledge Base

This document organizes xFrame5 framework knowledge for selective inclusion in prompt templates based on generation task type.

**Usage**: Include only relevant sections in prompts based on screen_type and requirements.

---

## Knowledge Categories

### 1. Core Architecture
**When to include**: Always include for all generation tasks
**Relevance**: Foundation concepts, component relationships

### 2. Component-Specific Knowledge
**When to include**: Based on components being used (Grid, Dataset, Popup, etc.)
**Relevance**: Detailed API, events, properties for specific components

### 3. Pattern Knowledge
**When to include**: Based on screen_type (list, detail, popup)
**Relevance**: Standard implementation patterns for common screen types

### 4. Server Integration
**When to include**: When generating transaction functions
**Relevance**: IO mapping, Java API, request/response handling

---

## 1. CORE ARCHITECTURE

### Framework Overview

xFrame5 is an HTML5-based UI development solution supporting "Any Device, Any Browser, Any Server Platform" with One Source Multi Use approach.

**Three Primary Systems**:
- User systems (UI layer)
- Server systems (WAS/MCA)
- Configuration management systems

### Component Architecture Layers

**UI Component Organization**:
- Grid: headers, rows, columns, items as discrete units
- Tab: tab items with separate content panels
- Code: unique IDs with code-to-display comment mappings

### Data & Communication Flow

**Screen Loading Stages**:
1. Parameter processing
2. Style loading
3. Global module initialization
4. Dataset preparation

**Screen Execution**:
- Event processing through Global Module variables/functions
- I/O mapping coordinates data assembly before server transmission
- WebSocket protocols for HTML5 UI ↔ xPlus5 functional layer communication

### Event Architecture

**Two Event Patterns**:
- **Simple events**: Direct action triggers
- **Return events**: Bidirectional data exchange between components and business logic

### Deployment Patterns

**Unified WEB/WAS**: Shared or local storage
**Separated WEB/WAS**: Corresponding storage options with failover/synchronization strategies

---

## 2. DATASET COMPONENT

**WHEN TO INCLUDE**: All screens that handle data (list, detail, form)

### Dataset Definition

Datasets are JavaScript modules for structured data management in xFrame5 screens.

### Global Dataset

**Purpose**: Application-wide data storage accessible across all screens

**Key Characteristics**:
- All screens can reference global dataset data
- Modifications in one screen visible to others
- Data persists across screen sessions

**Implementation Methods**:
1. xTranMap integration (drag-and-drop)
2. Component binding via `link_data` property
3. Script invocation: `[GlobalDatasetName].[FunctionName]()`

**Common Operations**:
- `getrowcount()` - returns number of rows
- `isglobalxdataset()` - validates if object is global dataset

**Template Location**: `/HTML5/SCREEN/GLOBAL/global_xdataset`

### Dataset Column Definition

**Required Properties**:
- Column ID
- Data type
- Size (for string types)

**XML Structure**:
```xml
<XDataSet id="ds_list">
    <columns>
        <column id="col_id" type="string" size="50"/>
        <column id="col_name" type="string" size="100"/>
        <column id="col_value" type="int"/>
    </columns>
</XDataSet>
```

### Java Server-Side API

**WHEN TO INCLUDE**: When generating transaction stub functions that call backend

#### Core Data Access Methods

**getData** - Retrieve request dataset values:
- String: `getData(String datasetName, String columnName, int recordIndex)`
- By index: `getData(String datasetName, int columnIndex, int recordIndex)`
- Typed variants: `getDoubleData()`, `getIntData()`, `getLongData()`, `getFloatData()`

**setData** - Populate response datasets before transmission to clients
- Supports column-name and index-based access

#### Record State Detection

Boolean methods classify incoming records:
- `isInsertRecord()` - determines if record is new
- `isUpdateRecord()` - determines if record is modified
- `isDeleteRecord()` - determines if record is marked for deletion

**Usage**: Enable selective processing based on change tracking from UI layer

#### Constructor Patterns

Four initialization approaches:
1. **Bidirectional**: HttpRequest + HttpResponse for stateless handlers
2. **Request-only**: Parses incoming data via XDATASET5 parameter
3. **Response-only**: Requires manual dataset/column configuration
4. **Minimal**: Defers setup to setter methods

#### Transaction Context Access

Methods expose request metadata for audit/routing:
- `getScreenNo()`
- `getTerminalIpAddress()`
- `getTransactionCode()`
- `getTransactionHeader()`
- `getTransactionMapId()`

#### Server Response Methods

- `returnData()` - transmits complete response datasets
- `returnPartData()` - streaming via "First-Row" methodology for large result sets
- `replaceHTMLTagFilter()` - reverses XSS entity encoding on received strings

---

## 3. GRID COMPONENT

**WHEN TO INCLUDE**: List screens, master-detail screens, any screen with tabular data

### Grid Structure

Grid consists of two main sections:
- **Header area** (`headertitle`)
- **Data area**

### XML Structure

```xml
<Grid id="grid_list" link_data="ds_list">
    <columns>
        <column title="ID" data_column="col_id" width="100"/>
        <column title="Name" data_column="col_name" width="200"/>
        <column title="Value" data_column="col_value" width="150"/>
    </columns>
</Grid>
```

### Key Properties

**Header-Related**:
- `title` - header labels
- `column` - column definitions

**Data-Related**:
- `data_type` - specifies data format
- `input_type` - defines interaction method
- `editable` - enables cell editing
- `text_horzalign` - text alignment
- `link_data` - dataset binding

**Grid-Level**:
- `linenumber_show` - displays row numbers
- `use_checkrow` - enables checkboxes

### Essential Grid API Methods

**Data Operations**:
- `getrowcount()` - returns number of rows
- `getcolumncount()` - returns number of columns
- `getitemtext(row, col)` - retrieves cell value
- `setitemtext(row, col, value)` - sets cell value

**Row Operations**:
- `additem()` - adds new item
- `addrow()` - adds new row
- `deleterow(row)` - deletes specific row
- `deleteall()` - clears all rows

### Grid Events (40+ events)

**WHEN TO INCLUDE**: When generating event handlers for grid interactions

#### Selection & Editing Events

- `on_itemselchange` - fires when cell focus changes (prev/current row/col indices)
- `on_itemeditshow` - triggered at edit start/end
- `on_itemeditcomplete` - called when editing concludes
- `on_itemeditvalidation` - before edit completion, returns 0-2 for navigation control
- `on_itemvaluechange` - fires during active editing as text changes
- `on_itemvaluechanged` - after edit completion with before/after values

#### Click & Interaction Events

- `on_itemclick` - cell click with button/image parameters
- `on_itemmousedown` - precedes click detection
- `on_itemdblclick` - double-click handling
- `on_itembtnclick` - button within editable cells
- `on_itempopbtnclick` - pop-button (dialog launcher) clicks
- `on_headerclick` / `on_headerdblclick` - column header interactions
- `on_headercheckclick` - select-all checkbox
- `on_statitemclick` / `on_statitemdblclick` - summary row/column interactions

#### Keyboard & Focus Events

- `on_prekeydown` - pre-processing, returns 0-1 to allow/prevent default
- `on_keydown` - post-processing with ctrl/shift/alt state, edit mode, position
- `on_focusin` / `on_focusout` - grid-level focus transitions

#### Drag-Drop Events

- `on_begindrag` - user begins drag with dragable=true, returns 1 to permit
- `on_enddrag` - completes internal drag-drop with source/target coordinates
- `on_dropcomplete` - external component dropped into grid
- `on_dropfiles` - files dragged from file explorer (provides File object array)

#### Sorting & Filtering Events

- `on_sortcomplete` - includes start timestamp for performance measurement
- `on_filtercomplete` - fires after filter application

#### File I/O Events

- `on_fileloadstart` - begin load with filename
- `on_fileload` - completion with result code, error details, row range, timing
- `on_filesavestart` / `on_filesave` - save lifecycle with success/error codes

#### Data Manipulation Events

- `on_paste` - pre-paste validation, return 1 to allow
- `on_pastecomplete` - post-paste with last-item flag
- `on_selectblock` - block selection with sum/average/count/min/max
- `on_validation` - data validation returns 1 (valid) or 0 (invalid)

#### Mouse & Context Events

- `on_mousein` / `on_mouseout` / `on_mousedown` / `on_click`
- `on_rclick` - right-click with grid and page coordinates
- `on_precontextmenu` - returns 1 to display, other values suppress

#### Structural Modification Events

- `on_columnwidthchange` - column resize with index
- `on_columnmove` - column reposition with source/destination indices

#### Checkbox & State Events

- `on_checkrowclick` - checkbox row interaction
- `on_checkrowchange` - state modification (0=unchecked, 1=checked, 2=partial)

#### Scroll Events

- `on_vscroll` / `on_hscroll` - vertical/horizontal scroll with position, prev position, scroll code, max-position flag

#### User Action Event

- `on_useraction` - generic action handler with action type and parameters

**Event Handler Pattern**:
- First parameter: grid instance
- Return values: 0=allow default, 1=prevent; for validation: 1=valid, 0=invalid

### Component Variants

Three grid types:
- **Standard grid** - basic tabular data
- **Multiline grid** - multiple lines per row
- **Tree grid** - hierarchical data

---

## 4. POPUP PATTERNS

**WHEN TO INCLUDE**: Detail screens, lookup popups, dialog-based interactions

### Popup Modes

- **Modal** - blocks interaction with parent screen
- **Modeless** - allows concurrent parent interaction

**Note**: General popups support both; portlet popups are modal-only

### Opening Popups

Use `loadpopup` API to invoke popup screens.

### Parameter Passing

Data flows between parent and popup using `extra_data` parameter:
- Values can be passed TO popup screens
- Values can be received FROM popup screens

### Return Value Handling

Child screens return data through three mechanisms:

1. **Field names** - direct variable assignment
2. **Function calls** - invoke parent methods
3. **Event callbacks** - `on_popupdestroy` event triggered when popup closes

### Implementation Files

Reference template includes:
- `popup_basic.xml` - structure
- `popup_basic.js` - behavior

---

## 5. IO MAPPING & TRANSACTIONS

**WHEN TO INCLUDE**: When generating transaction functions (fn_search, fn_save, fn_delete)

### Three Mapping Types

#### 1. Transaction Mapping

- **Format**: xDataset format
- **Protocol**: HTTP
- **Relationship**: N:N (datasets to services)
- **Features**:
  - Automatic control generation
  - Drag-and-drop mapping
  - All controls connected to dataset synchronize processing
  - Supports multiple datasets in single transaction

**Use when**: Standard CRUD operations with dataset-based data handling

#### 2. TranMap Mapping

- **Format**: Flat, XML, or JSON data
- **Protocol**: TCP/HTTP
- **Relationship**: 1:N mapping
- **Features**:
  - Direct intuitive data processing for flat formats
  - Automatic control generation
  - Supports multiple data formats

**Use when**: Need flat data handling clarity or non-dataset formats

#### 3. Tran I/O Map

- **Format**: Complex control bindings
- **Relationship**: 1:N (control to data)
- **Features**:
  - Multiple data bindings per control
  - No drag-and-drop mapping

**Use when**: Complex multi-data control scenarios

### Transaction Processing

All mapping types support:
- Header data processing
- Schema-based column generation
- Communication with document management servers

### Selection Criteria

Choose based on:
- Server type (WAS or MCA)
- Protocol requirements
- Transaction processing complexity

---

## 6. GLOBAL MODULES

**WHEN TO INCLUDE**: When generating utility functions or shared code

### Definition

Global modules are JavaScript modules accessible across all screens, consisting of:
- Global variables defined outside functions
- Function definitions

### Accessing Global Elements

**Functions**: `ModuleName.functionName()`
**Variables**: `ModuleName.variableName()`

### Key Characteristic

Global variables modified in one screen reflect changes across ALL screens.

### Cross-Module Access

Functions within global modules can:
- Invoke functions from other global modules
- Reference variables from other global modules

### Example Usage

```javascript
// In any screen:
SYSUtil.AddScreenRank()
```

**Purpose**: Shared utilities, constants, stateful data management

---

## 7. NAMING CONVENTIONS

**WHEN TO INCLUDE**: Always - critical for code generation

### Standard Prefixes

**Datasets**: `ds_` prefix
- `ds_list` - list data
- `ds_detail` - detail/form data
- `ds_search` - search criteria

**Grids**: `grid_` prefix
- `grid_list` - main list grid
- `grid_detail` - detail grid

**Functions**: `fn_` prefix
- `fn_search` - search/retrieve data
- `fn_save` - save/update data
- `fn_delete` - delete data
- `fn_add` - add new row

**Global Modules**: PascalCase
- `SYSUtil` - system utilities
- `CommonCode` - common code management

**Events**: `on_` prefix
- `on_itemclick` - grid item click
- `on_load` - screen load
- `on_popupdestroy` - popup destroy

---

## TASK-BASED KNOWLEDGE SELECTION

### For List Screen Generation

Include:
1. Core Architecture (brief)
2. Dataset Component (focus on column definition)
3. Grid Component (structure, properties, basic events)
4. IO Mapping (Transaction type)
5. Naming Conventions (all)

**Key Events**: `on_itemclick`, `on_itemdblclick`, `on_load`
**Key Functions**: `fn_search`, `fn_delete`, `fn_add`

### For Detail/Form Screen Generation

Include:
1. Core Architecture (brief)
2. Dataset Component (focus on column definition, data binding)
3. Popup Patterns (if detail is popup-based)
4. IO Mapping (Transaction type)
5. Naming Conventions (all)

**Key Events**: `on_load`, `on_popupdestroy` (if popup)
**Key Functions**: `fn_save`, `fn_delete`, `fn_load`

### For Popup Screen Generation

Include:
1. Popup Patterns (all)
2. Dataset Component (parameter passing)
3. Naming Conventions (all)

**Key Patterns**: Modal/modeless, extra_data, return mechanisms

### For Master-Detail Screen Generation

Include:
1. Core Architecture (data flow)
2. Dataset Component (all)
3. Grid Component (all including events)
4. IO Mapping (Transaction type)
5. Naming Conventions (all)

**Key Events**: `on_itemclick`, `on_itemselchange`, `on_itemdblclick`
**Key Functions**: `fn_search_master`, `fn_search_detail`, `fn_save`, `fn_delete`

### For Nested Grid Generation

Include:
1. Grid Component (all events, especially row selection)
2. Dataset Component (parent-child relationships)
3. Global Dataset (if shared across grids)

**Key Events**: `on_itemselchange`, `on_itemclick`

---

## VALIDATION REQUIREMENTS

**ALWAYS INCLUDE**: Critical for ensuring generated code quality

### XML Validation

- All Dataset IDs must start with `ds_`
- All Grid IDs must start with `grid_`
- Dataset columns must match Grid column bindings via `data_column` attribute
- Grid must have `link_data` property referencing valid dataset ID

### JavaScript Validation

- All function names must start with `fn_`
- All event handlers must start with `on_`
- Transaction functions must include error handling
- Dataset references must match XML definitions

### TODO Comment Rules

**CRITICAL**: When information is missing or unknown:
- NEVER make up API endpoints
- NEVER guess server URLs
- ALWAYS add TODO comments with specific placeholders

**Example**:
```javascript
// TODO: Replace with actual API endpoint
var url = "/api/placeholder/search";
```

---

## COMMON PATTERNS

### Standard Transaction Function Structure

```javascript
this.fn_search = function() {
    // TODO: Set transaction URL
    var tranUrl = "/api/placeholder/search";

    // Set transaction info
    // Call transaction
    // Handle callback
};
```

### Standard Grid Click Handler

```javascript
grid_list.on_itemdblclick = function(row, col, buttonClick, imageIndex) {
    var ds = ds_list;
    var selectedValue = ds.getitemtext(row, "col_id");

    // TODO: Implement detail screen logic
};
```

### Standard Popup Return Handler

```javascript
this.on_popupdestroy = function(returnValue) {
    if (returnValue) {
        // Process returned data
        // Refresh parent screen
    }
};
```

---

## VERSION NOTES

- **Source**: xFrame5 TechNet (https://technet.softbase.co.kr/wiki)
- **Last Updated**: 2025-12-28
- **Coverage**: Architecture, Dataset, Grid, Popup, IO Mapping, Global Modules
- **Limitations**: Many specific API pages incomplete in official documentation
