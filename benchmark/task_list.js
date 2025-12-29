/**
 * Task List Screen JavaScript
 * Generated as benchmark for code quality evaluation
 *
 * Features:
 * - Search/filter tasks
 * - Create new task via popup
 * - Edit existing task via popup
 * - Delete selected tasks
 * - Grid double-click opens editor
 */

// ============================================
// SCREEN INITIALIZATION
// ============================================

/**
 * Screen load event handler
 * Called when screen is first loaded
 */
this.on_load = function() {
    // Initialize screen
    fn_init();

    // Load initial task list
    fn_search();
};

/**
 * Initialize screen components
 */
this.fn_init = function() {
    // Set focus to search field
    field_search_title.setfocus();

    // Initialize combobox selections
    cbo_search_status.setselectedcode(0);
    cbo_search_priority.setselectedcode(0);
};

// ============================================
// SEARCH FUNCTIONS
// ============================================

/**
 * Search/Query tasks based on filter criteria
 * Called by Query button and on_load
 */
this.fn_search = function() {
    // Collect search parameters
    var searchTitle = field_search_title.getvalue();
    var searchStatus = cbo_search_status.getselectedcode();
    var searchPriority = cbo_search_priority.getselectedcode();

    // Set search parameters to dataset
    ds_search.deleteall();
    ds_search.addrow();
    ds_search.setitemtext(0, "TITLE", searchTitle);
    ds_search.setitemtext(0, "STATUS", searchStatus);
    ds_search.setitemtext(0, "PRIORITY", searchPriority);

    // TODO: Replace with actual API endpoint
    var tranUrl = "/api/tasks/search";

    // Set transaction info
    var tranInfo = {
        url: tranUrl,
        method: "POST",
        input: "ds_search",
        output: "ds_list",
        callback: "fn_search_callback"
    };

    // Execute transaction
    // TODO: Implement actual transaction call
    // xcomm.execute(tranInfo);

    // For demonstration, show message
    console.log("fn_search called with params:", {
        title: searchTitle,
        status: searchStatus,
        priority: searchPriority
    });

    // Update total count display
    fn_updateTotalCount();
};

/**
 * Callback function for search transaction
 * @param {Object} result - Transaction result
 */
this.fn_search_callback = function(result) {
    if (result.success) {
        // Data is automatically bound to ds_list
        fn_updateTotalCount();
    } else {
        // Show error message
        alert("Failed to search tasks: " + result.message);
    }
};

/**
 * Reset search filters to default
 */
this.fn_reset = function() {
    // Clear search field
    field_search_title.setvalue("");

    // Reset comboboxes to "All"
    cbo_search_status.setselectedcode(0);
    cbo_search_priority.setselectedcode(0);

    // Re-search with cleared filters
    fn_search();
};

/**
 * Update total count display
 */
this.fn_updateTotalCount = function() {
    var rowCount = ds_list.getrowcount();
    txt_total_count.settext("Total: " + rowCount + " items");
};

// ============================================
// CRUD FUNCTIONS
// ============================================

/**
 * Create new task - opens editor popup in create mode
 * Called by Create button
 */
this.fn_create = function() {
    // Open task editor popup in create mode
    var popupParams = {
        mode: "create",
        taskId: null
    };

    // TODO: Replace with actual popup screen path
    var popupUrl = "/screens/task_editor.xml";

    loadpopup({
        url: popupUrl,
        width: 600,
        height: 500,
        modal: true,
        title: "Create New Task",
        extra_data: popupParams,
        on_popupdestroy: "fn_onEditorClose"
    });
};

/**
 * Edit selected task - opens editor popup in edit mode
 * Called by Edit button
 */
this.fn_edit = function() {
    // Get selected row
    var selectedRow = grid_list.getfocusedrowidx();

    if (selectedRow < 0) {
        alert("Please select a task to edit.");
        return;
    }

    // Get task ID from selected row
    var taskId = ds_list.getitemtext(selectedRow, "TASK_ID");

    if (!taskId) {
        alert("Cannot identify selected task.");
        return;
    }

    // Open task editor popup in edit mode
    var popupParams = {
        mode: "edit",
        taskId: taskId
    };

    // TODO: Replace with actual popup screen path
    var popupUrl = "/screens/task_editor.xml";

    loadpopup({
        url: popupUrl,
        width: 600,
        height: 500,
        modal: true,
        title: "Edit Task",
        extra_data: popupParams,
        on_popupdestroy: "fn_onEditorClose"
    });
};

/**
 * Delete selected tasks
 * Called by Delete button
 */
this.fn_delete = function() {
    // Get checked rows
    var checkedRows = grid_list.getcheckedrowidx();

    if (!checkedRows || checkedRows.length === 0) {
        // If no checked rows, try selected row
        var selectedRow = grid_list.getfocusedrowidx();
        if (selectedRow < 0) {
            alert("Please select at least one task to delete.");
            return;
        }
        checkedRows = [selectedRow];
    }

    // Confirm deletion
    var count = checkedRows.length;
    var confirmMsg = "Are you sure you want to delete " + count + " task(s)?";

    if (!confirm(confirmMsg)) {
        return;
    }

    // Collect task IDs to delete
    var taskIds = [];
    for (var i = 0; i < checkedRows.length; i++) {
        var rowIdx = checkedRows[i];
        var taskId = ds_list.getitemtext(rowIdx, "TASK_ID");
        if (taskId) {
            taskIds.push(taskId);
        }
    }

    if (taskIds.length === 0) {
        alert("No valid tasks to delete.");
        return;
    }

    // TODO: Replace with actual API endpoint
    var tranUrl = "/api/tasks/delete";

    // Prepare delete request
    var deleteParams = {
        taskIds: taskIds
    };

    // TODO: Implement actual delete transaction
    // xcomm.execute({
    //     url: tranUrl,
    //     method: "POST",
    //     data: deleteParams,
    //     callback: "fn_delete_callback"
    // });

    console.log("fn_delete called with taskIds:", taskIds);

    // For demonstration, remove rows from grid
    // In production, refresh after server confirmation
    fn_delete_callback({ success: true });
};

/**
 * Callback function for delete transaction
 * @param {Object} result - Transaction result
 */
this.fn_delete_callback = function(result) {
    if (result.success) {
        // Refresh the list
        fn_search();
        alert("Task(s) deleted successfully.");
    } else {
        alert("Failed to delete task(s): " + result.message);
    }
};

// ============================================
// POPUP CALLBACK FUNCTIONS
// ============================================

/**
 * Callback when task editor popup is closed
 * @param {Object} returnValue - Data returned from popup
 */
this.fn_onEditorClose = function(returnValue) {
    if (returnValue && returnValue.saved) {
        // Task was saved, refresh the list
        fn_search();

        // Show success message
        var action = returnValue.mode === "create" ? "created" : "updated";
        alert("Task " + action + " successfully.");
    }
    // If popup was cancelled, no action needed
};

// ============================================
// GRID EVENT HANDLERS
// ============================================

/**
 * Grid double-click event handler
 * Opens task editor in edit mode
 *
 * @param {Object} objInst - Grid instance
 * @param {number} nRow - Row index
 * @param {number} nColumn - Column index
 * @param {number} buttonClick - Mouse button (1=left, 2=right)
 * @param {number} imageIndex - Image index if cell contains image
 */
this.grid_list_on_itemdblclick = function(objInst, nRow, nColumn, buttonClick, imageIndex) {
    // Only respond to left-button double-click
    if (buttonClick !== 1) {
        return;
    }

    // Ignore header double-click (row -1)
    if (nRow < 0) {
        return;
    }

    // Open editor for double-clicked row
    var taskId = ds_list.getitemtext(nRow, "TASK_ID");

    if (taskId) {
        var popupParams = {
            mode: "edit",
            taskId: taskId
        };

        // TODO: Replace with actual popup screen path
        var popupUrl = "/screens/task_editor.xml";

        loadpopup({
            url: popupUrl,
            width: 600,
            height: 500,
            modal: true,
            title: "Edit Task",
            extra_data: popupParams,
            on_popupdestroy: "fn_onEditorClose"
        });
    }
};

/**
 * Grid selection change event handler
 * Updates button states based on selection
 *
 * @param {Object} objInst - Grid instance
 * @param {number} nPrevRow - Previous row index
 * @param {number} nPrevColumn - Previous column index
 * @param {number} nRow - Current row index
 * @param {number} nColumn - Current column index
 */
this.grid_list_on_itemselchange = function(objInst, nPrevRow, nPrevColumn, nRow, nColumn) {
    // Enable/disable edit button based on selection
    var hasSelection = (nRow >= 0);

    // TODO: Enable/disable button styling if framework supports
    // btn_edit.setenabled(hasSelection);
    // btn_delete.setenabled(hasSelection);

    if (hasSelection) {
        // Log selected task for debugging
        var taskId = ds_list.getitemtext(nRow, "TASK_ID");
        var taskTitle = ds_list.getitemtext(nRow, "TASK_TITLE");
        console.log("Selected task:", taskId, taskTitle);
    }
};

// ============================================
// UTILITY FUNCTIONS
// ============================================

/**
 * Format date for display
 * @param {string} dateStr - Date string from server
 * @returns {string} Formatted date string
 */
this.fn_formatDate = function(dateStr) {
    if (!dateStr) {
        return "";
    }

    try {
        var date = new Date(dateStr);
        var year = date.getFullYear();
        var month = ("0" + (date.getMonth() + 1)).slice(-2);
        var day = ("0" + date.getDate()).slice(-2);
        return year + "-" + month + "-" + day;
    } catch (e) {
        return dateStr;
    }
};

/**
 * Get status display name from code
 * @param {string} statusCode - Status code
 * @returns {string} Status display name
 */
this.fn_getStatusName = function(statusCode) {
    var statusMap = {
        "01": "Pending",
        "02": "In Progress",
        "03": "Completed",
        "04": "Cancelled"
    };
    return statusMap[statusCode] || "Unknown";
};

/**
 * Get priority display name from code
 * @param {string} priorityCode - Priority code
 * @returns {string} Priority display name
 */
this.fn_getPriorityName = function(priorityCode) {
    var priorityMap = {
        "1": "High",
        "2": "Medium",
        "3": "Low"
    };
    return priorityMap[priorityCode] || "Unknown";
};
