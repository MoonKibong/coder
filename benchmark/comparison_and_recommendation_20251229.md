  Quality Gap Analysis

  | Aspect          | LLM Output (llama3.1) | Benchmark                          | Gap             |
  |-----------------|-----------------------|------------------------------------|-----------------|
  | Components      | 4 (grid, 3 buttons)   | 40+ (panels, fields, combos, etc.) | Missing 90%     |
  | Dataset columns | 3                     | 11                                 | Missing context |
  | Positioning     | None                  | Full x,y,width,height              | Broken layout   |
  | Event syntax    | onclick="fn_search"   | on_click="eventfunc:fn_search()"   | Wrong syntax    |
  | Edit button     | Missing               | Present                            | Incomplete      |
  | Editor popup    | Not mentioned         | Full integration                   | No association  |
  | JavaScript      | None                  | 650+ lines                         | No behavior     |

  Root Causes & Fixes

  1. Prompt Template Needs Concrete XML Examples

  Current prompt likely has vague instructions. Add explicit syntax:

  system_prompt: |
    # EXACT COMPONENT SYNTAX (use these patterns)

    ## Button (MUST include all attributes):
    <pushbutton control_id="1" name="btn_query"
                x="10" y="10" width="100" height="30"
                text="Query"
                font="Malgun Gothic,9,0,0,0,0"
                on_click="eventfunc:fn_search()"/>

    ## Grid with dataset binding:
    <grid control_id="0" name="grid_list"
          x="0" y="0" width="800" height="400"
          link_data="ds_list"
          linenumber_show="1"
          use_checkrow="1"
          version="1.1"
          on_itemdblclick="eventfunc:grid_list_on_itemdblclick(objInst, nRow, nColumn, buttonClick, imageIndex)">

  2. Enforce Both XML and JS Output

  Add explicit output format instruction:

  system_prompt: |
    # OUTPUT FORMAT (MANDATORY)
    You MUST generate BOTH files:

    --- XML ---
    [Complete screen XML with all components]

    --- JS ---
    [Complete JavaScript with all functions]

    NEVER skip JavaScript. Every button needs a corresponding fn_* function.

  3. Add Component Checklist for Screen Types

  system_prompt: |
    # LIST SCREEN REQUIREMENTS (check all)
    - [ ] Dataset for list data (ds_list)
    - [ ] Dataset for search criteria (ds_search)
    - [ ] Search panel with filters
    - [ ] Button panel with CRUD buttons
    - [ ] Grid panel with columns matching dataset
    - [ ] All buttons have on_click handlers
    - [ ] Grid has on_itemdblclick for detail popup
    - [ ] JavaScript has fn_search, fn_create, fn_edit, fn_delete
    - [ ] Popup callback fn_onEditorClose for refresh

  4. Add Popup Association Pattern

  The LLM didn't know how to associate editor popup. Add this pattern:

  system_prompt: |
    # POPUP ASSOCIATION PATTERN
    When user requests "associate editor with create/edit button":

    Create button opens popup in CREATE mode:
    this.fn_create = function() {
        loadpopup({
            url: "/screens/task_editor.xml",
            modal: true,
            extra_data: { mode: "create", taskId: null },
            on_popupdestroy: "fn_onEditorClose"
        });
    };

    Edit button opens popup in EDIT mode:
    this.fn_edit = function() {
        var taskId = ds_list.getitemtext(selectedRow, "TASK_ID");
        loadpopup({
            url: "/screens/task_editor.xml",
            modal: true,
            extra_data: { mode: "edit", taskId: taskId },
            on_popupdestroy: "fn_onEditorClose"
        });
    };

  5. Include Knowledge Base in Prompt

  Your XFRAME5_XML_PATTERNS.md should be included in the system prompt:

  // In prompt_compiler.rs
  fn compile_prompt(request: &GenerateRequest) -> String {
      let knowledge = load_relevant_knowledge(request.screen_type);
      let template = load_template(request.product);

      format!("{}\n\n{}\n\n{}",
          template.system_prompt,
          knowledge,  // XML patterns from knowledge base
          request.user_input
      )
  }

  6. Model Capability Issue

  llama3.1:latest (8B) may lack capacity for complex structured output. Consider:

  | Model           | Parameters | Expected Quality                 |
  |-----------------|------------|----------------------------------|
  | llama3.1:8b     | 8B         | Basic structure, missing details |
  | qwen3-coder:30b | 30B        | Good structure, some gaps        |
  | codestral:22b   | 22B        | Good for code generation         |
  | llama3.1:70b    | 70B        | Near-benchmark quality           |
  | claude-opus     | -          | Benchmark level                  |

  7. Add Validation Rules to Prompt

  system_prompt: |
    # VALIDATION RULES (MUST follow)
    1. Every component MUST have x, y, width, height
    2. Every button MUST have on_click="eventfunc:fn_name()"
    3. Grid MUST have version="1.1"
    4. Dataset IDs MUST start with ds_
    5. Grid names MUST start with grid_
    6. NEVER use onclick (wrong) - use on_click (correct)
    7. Include TODO comments for unknown API endpoints

  Recommended Prompt Template Update

  name: xframe5-list-v2
  version: 2
  system_prompt: |
    You are an expert xFrame5 code generator.

    # CRITICAL RULES
    1. Generate BOTH XML and JavaScript files
    2. Use EXACT syntax patterns shown below
    3. Include ALL required component attributes
    4. Add TODO placeholders for unknown values

    # XML PATTERNS (from XFRAME5_XML_PATTERNS.md)
    [Include 50-100 lines of concrete examples]

    # JAVASCRIPT PATTERNS
    [Include fn_search, fn_create, fn_edit, fn_delete, popup callbacks]

    # OUTPUT FORMAT
    --- XML ---
    <?xml version="1.0" encoding="UTF-8"?>
    <screen id="SCREEN_..." ...>
    ...
    </screen>

    --- JS ---
    this.on_load = function() { ... };
    this.fn_search = function() { ... };
    ...

  Summary: Priority Improvements

  1. Add concrete XML/JS examples to prompt template (highest impact)
  2. Enforce dual output (XML + JS mandatory)
  3. Include knowledge base content in prompts
  4. Add component checklist for completeness
  5. Consider larger model (30B+ for better quality)
  6. Add post-generation validation to catch syntax errors

  Would you like me to update the xframe5-list prompt template in the database with these improvements?