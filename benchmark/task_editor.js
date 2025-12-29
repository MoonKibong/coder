/**
 * Task Editor Popup Screen JavaScript
 * Generated as benchmark for code quality evaluation
 *
 * Features:
 * - Create new task or edit existing task
 * - Form validation before save
 * - Return saved data to parent screen
 * - Cancel without saving
 */

// ============================================
// SCREEN VARIABLES
// ============================================

// Mode: "create" or "edit"
var g_mode = "create";

// Task ID for edit mode
var g_taskId = null;

// ============================================
// SCREEN INITIALIZATION
// ============================================

/**
 * Screen load event handler
 * Called when popup screen is loaded
 */
this.on_load = function() {
    // Get parameters passed from parent screen
    var extraData = screen.getextradata();

    if (extraData) {
        g_mode = extraData.mode || "create";
        g_taskId = extraData.taskId || null;
    }

    // Initialize screen based on mode
    fn_init();
};

/**
 * Initialize screen based on mode (create/edit)
 */
this.fn_init = function() {
    // Update popup title based on mode
    if (g_mode === "create") {
        txt_popup_title.settext("Create New Task");

        // Hide Task ID field in create mode
        txt_task_id_label.sethidden(true);
        field_task_id.sethidden(true);

        // Set default values for new task
        fn_setDefaultValues();

        // Set focus to title field
        field_title.setfocus();

    } else if (g_mode === "edit") {
        txt_popup_title.settext("Edit Task");

        // Show Task ID field in edit mode (readonly)
        txt_task_id_label.sethidden(false);
        field_task_id.sethidden(false);

        // Load task data from server
        fn_loadTaskData(g_taskId);
    }

    // Clear any previous error messages
    fn_clearError();
};

/**
 * Set default values for new task
 */
this.fn_setDefaultValues = function() {
    // Initialize dataset with empty row
    ds_detail.deleteall();
    ds_detail.addrow();

    // Set default status to "Pending"
    cbo_status.setselectedcode(0); // Index 0 = "01" (Pending)

    // Set default priority to "Medium"
    cbo_priority.setselectedcode(1); // Index 1 = "2" (Medium)

    // Clear other fields
    field_title.setvalue("");
    field_desc.setvalue("");
    field_assignee.setvalue("");
    field_due_date.setvalue("");
};

/**
 * Load task data for edit mode
 * @param {string} taskId - Task ID to load
 */
this.fn_loadTaskData = function(taskId) {
    if (!taskId) {
        fn_showError("Task ID is missing.");
        return;
    }

    // TODO: Replace with actual API endpoint
    var tranUrl = "/api/tasks/" + taskId;

    // TODO: Implement actual transaction call
    // xcomm.execute({
    //     url: tranUrl,
    //     method: "GET",
    //     output: "ds_detail",
    //     callback: "fn_loadTaskData_callback"
    // });

    console.log("fn_loadTaskData called for taskId:", taskId);

    // For demonstration, simulate loaded data
    fn_loadTaskData_callback({
        success: true,
        data: {
            TASK_ID: taskId,
            TASK_TITLE: "Sample Task " + taskId,
            TASK_DESC: "This is a sample task description for demonstration purposes.",
            STATUS: "02",
            PRIORITY: "2",
            ASSIGNEE: "John Doe",
            DUE_DATE: "2025-01-15"
        }
    });
};

/**
 * Callback function for load task data transaction
 * @param {Object} result - Transaction result
 */
this.fn_loadTaskData_callback = function(result) {
    if (result.success) {
        // Data is automatically bound to ds_detail if using standard transaction
        // For manual population (demonstration):
        if (result.data) {
            ds_detail.deleteall();
            ds_detail.addrow();

            ds_detail.setitemtext(0, "TASK_ID", result.data.TASK_ID);
            ds_detail.setitemtext(0, "TASK_TITLE", result.data.TASK_TITLE);
            ds_detail.setitemtext(0, "TASK_DESC", result.data.TASK_DESC);
            ds_detail.setitemtext(0, "STATUS", result.data.STATUS);
            ds_detail.setitemtext(0, "PRIORITY", result.data.PRIORITY);
            ds_detail.setitemtext(0, "ASSIGNEE", result.data.ASSIGNEE);
            ds_detail.setitemtext(0, "DUE_DATE", result.data.DUE_DATE);

            // Update form controls
            field_task_id.setvalue(result.data.TASK_ID);
            field_title.setvalue(result.data.TASK_TITLE);
            field_desc.setvalue(result.data.TASK_DESC);
            field_assignee.setvalue(result.data.ASSIGNEE);
            field_due_date.setvalue(result.data.DUE_DATE);

            // Set combobox selections by code value
            fn_setComboByCode(cbo_status, ds_status, result.data.STATUS);
            fn_setComboByCode(cbo_priority, ds_priority, result.data.PRIORITY);
        }

        // Set focus to title field
        field_title.setfocus();

    } else {
        fn_showError("Failed to load task data: " + result.message);
    }
};

// ============================================
// SAVE FUNCTIONS
// ============================================

/**
 * Save task data
 * Called by Save button
 */
this.fn_save = function() {
    // Validate form
    if (!fn_validate()) {
        return;
    }

    // Collect form data
    var taskData = fn_collectFormData();

    // TODO: Replace with actual API endpoint
    var tranUrl = (g_mode === "create")
        ? "/api/tasks"
        : "/api/tasks/" + g_taskId;

    var tranMethod = (g_mode === "create") ? "POST" : "PUT";

    // TODO: Implement actual transaction call
    // xcomm.execute({
    //     url: tranUrl,
    //     method: tranMethod,
    //     data: taskData,
    //     callback: "fn_save_callback"
    // });

    console.log("fn_save called with data:", taskData);

    // For demonstration, simulate successful save
    fn_save_callback({ success: true });
};

/**
 * Collect form data into object
 * @returns {Object} Task data object
 */
this.fn_collectFormData = function() {
    var taskData = {
        TASK_ID: (g_mode === "edit") ? g_taskId : null,
        TASK_TITLE: field_title.getvalue(),
        TASK_DESC: field_desc.getvalue(),
        STATUS: fn_getComboCode(cbo_status, ds_status),
        PRIORITY: fn_getComboCode(cbo_priority, ds_priority),
        ASSIGNEE: field_assignee.getvalue(),
        DUE_DATE: field_due_date.getvalue()
    };

    return taskData;
};

/**
 * Callback function for save transaction
 * @param {Object} result - Transaction result
 */
this.fn_save_callback = function(result) {
    if (result.success) {
        // Close popup and return success to parent
        var returnValue = {
            saved: true,
            mode: g_mode,
            taskId: (g_mode === "edit") ? g_taskId : result.taskId
        };

        closepopup(returnValue);

    } else {
        fn_showError("Failed to save task: " + result.message);
    }
};

// ============================================
// VALIDATION FUNCTIONS
// ============================================

/**
 * Validate form data
 * @returns {boolean} True if valid, false otherwise
 */
this.fn_validate = function() {
    // Clear previous errors
    fn_clearError();

    // Validate Title (required)
    var title = field_title.getvalue();
    if (!title || title.trim() === "") {
        fn_showError("Title is required.");
        field_title.setfocus();
        return false;
    }

    // Validate Title length
    if (title.length > 50) {
        fn_showError("Title must be 50 characters or less.");
        field_title.setfocus();
        return false;
    }

    // Validate Status (required)
    var statusIndex = cbo_status.getselectedindex();
    if (statusIndex < 0) {
        fn_showError("Status is required.");
        cbo_status.setfocus();
        return false;
    }

    // Validate Priority (required)
    var priorityIndex = cbo_priority.getselectedindex();
    if (priorityIndex < 0) {
        fn_showError("Priority is required.");
        cbo_priority.setfocus();
        return false;
    }

    // Validate Due Date format (if provided)
    var dueDate = field_due_date.getvalue();
    if (dueDate && !fn_isValidDate(dueDate)) {
        fn_showError("Due Date format is invalid.");
        field_due_date.setfocus();
        return false;
    }

    return true;
};

/**
 * Validate date format (yyyy-MM-dd)
 * @param {string} dateStr - Date string to validate
 * @returns {boolean} True if valid date format
 */
this.fn_isValidDate = function(dateStr) {
    if (!dateStr) {
        return true; // Empty is valid (optional field)
    }

    // Check format: yyyy-MM-dd
    var regex = /^\d{4}-\d{2}-\d{2}$/;
    if (!regex.test(dateStr)) {
        return false;
    }

    // Check if date is valid
    var parts = dateStr.split("-");
    var year = parseInt(parts[0], 10);
    var month = parseInt(parts[1], 10);
    var day = parseInt(parts[2], 10);

    if (month < 1 || month > 12) {
        return false;
    }

    var daysInMonth = new Date(year, month, 0).getDate();
    if (day < 1 || day > daysInMonth) {
        return false;
    }

    return true;
};

// ============================================
// ERROR HANDLING
// ============================================

/**
 * Show error message
 * @param {string} message - Error message to display
 */
this.fn_showError = function(message) {
    txt_error_message.settext(message);
    txt_error_message.sethidden(false);
};

/**
 * Clear error message
 */
this.fn_clearError = function() {
    txt_error_message.settext("");
    txt_error_message.sethidden(true);
};

// ============================================
// CANCEL FUNCTION
// ============================================

/**
 * Cancel editing and close popup
 * Called by Cancel button
 */
this.fn_cancel = function() {
    // Check if there are unsaved changes
    if (fn_hasChanges()) {
        var confirmMsg = "You have unsaved changes. Are you sure you want to cancel?";
        if (!confirm(confirmMsg)) {
            return;
        }
    }

    // Close popup without saving
    var returnValue = {
        saved: false,
        mode: g_mode
    };

    closepopup(returnValue);
};

/**
 * Check if form has unsaved changes
 * @returns {boolean} True if there are changes
 */
this.fn_hasChanges = function() {
    // For create mode, check if any field has value
    if (g_mode === "create") {
        var title = field_title.getvalue();
        var desc = field_desc.getvalue();
        var assignee = field_assignee.getvalue();

        return (title && title.trim() !== "") ||
               (desc && desc.trim() !== "") ||
               (assignee && assignee.trim() !== "");
    }

    // For edit mode, compare with original values
    // TODO: Implement proper change detection by comparing with loaded values
    return true; // Assume changes for safety
};

// ============================================
// COMBOBOX EVENT HANDLERS
// ============================================

/**
 * Status combobox change event handler
 * @param {Object} objInst - Combobox instance
 * @param {number} nPrevIndex - Previous selected index
 * @param {number} nIndex - Current selected index
 */
this.cbo_status_on_change = function(objInst, nPrevIndex, nIndex) {
    // Update dataset with selected value
    var code = fn_getComboCode(objInst, ds_status);
    ds_detail.setitemtext(0, "STATUS", code);

    console.log("Status changed to:", code);
};

// ============================================
// UTILITY FUNCTIONS
// ============================================

/**
 * Get code value from combobox based on selected index
 * @param {Object} combo - Combobox control
 * @param {Object} dataset - Bound dataset
 * @returns {string} Code value
 */
this.fn_getComboCode = function(combo, dataset) {
    var index = combo.getselectedindex();
    if (index >= 0 && index < dataset.getrowcount()) {
        return dataset.getitemtext(index, "CODE");
    }
    return "";
};

/**
 * Set combobox selection by code value
 * @param {Object} combo - Combobox control
 * @param {Object} dataset - Bound dataset
 * @param {string} code - Code value to select
 */
this.fn_setComboByCode = function(combo, dataset, code) {
    var rowCount = dataset.getrowcount();
    for (var i = 0; i < rowCount; i++) {
        var rowCode = dataset.getitemtext(i, "CODE");
        if (rowCode === code) {
            combo.setselectedcode(i);
            return;
        }
    }

    // Default to first item if code not found
    if (rowCount > 0) {
        combo.setselectedcode(0);
    }
};

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
