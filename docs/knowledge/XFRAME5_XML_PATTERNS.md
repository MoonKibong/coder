# xFrame5 XML Patterns Reference

**Source**: https://technet.softbase.co.kr/project/template/template.html
**Date**: 2025-12-28
**Purpose**: Concrete XML syntax patterns from official xFrame5 templates

This document provides actual XML syntax examples extracted from xFrame5 template files for code generation.

---

## XDATASET - Dataset Definitions

### Basic XLinkDataset (Column String Format)

```xml
<xlinkdataset id="DS_PRODUCT" desc=""
  columns="PROD_CODE:&quot;상품코드&quot;:6:&quot;205490&#x0A;101830&#x0A;121894&#x0A;&quot;:&quot;screen.XDatasetSetCallback&quot;;
           PROD_NAME:&quot;상품명&quot;:10:&quot;상품1&#x0A;상품2&#x0A;상품3&#x0A;&quot;:&quot;screen.XDatasetSetCallback&quot;;
           PRICE:&quot;가격&quot;:8:&quot;10000&#x0A;20000&#x0A;30000&#x0A;&quot;:&quot;screen.XDatasetSetCallback&quot;"/>
```

**Column Format**: `COLUMN_NAME:"Display Name":Width:"Default Data":"Callback Function"`

**Key Elements**:
- `id` - Dataset unique identifier (use `ds_` prefix)
- `desc` - Description (optional)
- `columns` - Semicolon-separated column definitions
- `&#x0A;` - Line break character for row data
- `&quot;` - Quotation mark HTML entity

### XDataset with Columns Element (Structured Format)

```xml
<xlinkdataset id="DS_LIST" desc="">
  <columns>
    Column_01:::&quot;11&#x0A;21&#x0A;31&#x0A;&quot;:;
    Column_02:::&quot;12&#x0A;22&#x0A;32&#x0A;&quot;:;
    Column_03:::&quot;13&#x0A;23&#x0A;33&#x0A;&quot;:
  </columns>
</xlinkdataset>
```

### Dataset for Server Communication

```xml
<xdataset id="ds_search" desc="Search Criteria">
  <!-- No columns defined - populated programmatically -->
</xdataset>

<xdataset id="ds_result" desc="Search Results">
  <!-- Columns populated from server response -->
</xdataset>
```

**Usage Notes**:
- Use `xlinkdataset` for static/demo data with embedded values
- Use `xdataset` for server-bound datasets without embedded data
- Column data in `xlinkdataset` uses `&#x0A;` for row separation

---

## GRID - Data Grid Component

### Basic Grid Structure

```xml
<grid control_id="0" name="grid_list"
      x="6" y="612" right="434" bottom="14"
      width="372" height="234"
      linenumber_show="1"
      version="1.1">

  <column>
    <header title="숫자"/>
    <data width="87"
          text_horzalign="2"
          data_type="0"
          pattern="-ZZ,ZZZ,ZZZ,ZZZ,ZZ9"
          prefilldata="1234"/>
  </column>

  <column>
    <header title="문자"/>
    <data width="100"
          text_horzalign="1"
          data_type="2"
          prefilldata="Text Value"/>
  </column>

</grid>
```

**Key Properties**:
- `control_id` - Unique control identifier
- `name` - Grid reference name (use `grid_` prefix)
- `x`, `y`, `right`, `bottom` - Positioning
- `width`, `height` - Dimensions
- `linenumber_show` - Show row numbers ("1" = yes)
- `version` - Grid version

**Column Structure**:
- `<header title="..."/>` - Column header
- `<data .../>` - Column data configuration

**Data Properties**:
- `width` - Column width in pixels
- `text_horzalign` - Horizontal alignment (0=left, 1=center, 2=right)
- `data_type` - Data type (0=numeric, 2=string)
- `pattern` - Number format pattern
- `prefilldata` - Static demo data

### Grid with Dataset Binding

```xml
<xlinkdataset id="DS_PRODUCT" desc=""
  columns="PROD_CODE:&quot;상품코드&quot;:6:&quot;&quot;:&quot;&quot;;
           PROD_NAME:&quot;상품명&quot;:10:&quot;&quot;:&quot;&quot;;
           PRICE:&quot;가격&quot;:8:&quot;&quot;:&quot;&quot;"/>

<grid control_id="0" name="grid_list"
      link_data="DS_PRODUCT"
      width="400" height="300">

  <column>
    <header title="상품코드"/>
    <data name="PROD_CODE"
          link_data="DS_PRODUCT:PROD_CODE"
          width="100"
          text_horzalign="1"
          data_type="2"/>
  </column>

  <column>
    <header title="상품명"/>
    <data name="PROD_NAME"
          link_data="DS_PRODUCT:PROD_NAME"
          width="150"
          text_horzalign="1"
          data_type="2"/>
  </column>

  <column>
    <header title="가격"/>
    <data name="PRICE"
          link_data="DS_PRODUCT:PRICE"
          width="100"
          text_horzalign="2"
          data_type="0"
          pattern="#,##0"/>
  </column>

</grid>
```

**Dataset Binding**:
- Grid level: `link_data="DS_PRODUCT"` - Links grid to dataset
- Column level: `link_data="DS_PRODUCT:COLUMN_NAME"` - Links column to dataset column
- Column level: `name="COLUMN_NAME"` - Column identifier

### Editable Grid

```xml
<grid control_id="5" name="grid_edit"
      editable="1"
      link_data="DS_EDIT"
      on_itemeditshow="eventfunc:grid_edit_on_itemeditshow(objInst, nRow, nColumn, bShowEdit)"
      on_itemeditcomplete="eventfunc:grid_edit_on_itemeditcomplete(objInst, nRow, nColumn, strPrevItemText)"
      on_itemvaluechanged="eventfunc:grid_edit_on_itemvaluechanged(objInst, nRow, nColumn, strPrevItemText, strItemText)">

  <column>
    <header title="편집가능"/>
    <data name="COL_EDIT"
          link_data="DS_EDIT:COL_EDIT"
          width="150"
          text_horzalign="1"
          editable="1"
          data_type="2"/>
  </column>

</grid>
```

**Editable Properties**:
- Grid level: `editable="1"` - Enables editing
- Column level: `editable="1"` - Enables column editing
- `edit_focus_select` - Cursor positioning when edit starts

**Edit Events**:
- `on_itemeditshow` - Triggered when editing begins
- `on_itemeditcomplete` - Triggered when editing ends
- `on_itemvaluechanged` - Triggered when value changes

### Grid with Checkbox Column

```xml
<grid control_id="0" name="grid_check"
      use_checkrow="1"
      checkrow_hd_backcolor="00FCF5EF"
      checkrow_statrowstyle="1">

  <column>
    <header title="체크박스"
            check_leftmargin="0"
            check_rightmargin="0"/>
    <data name="CHECK_COL"
          width="150"
          text_horzalign="1"
          editable="1"
          input_type="1"
          prefilldata="1&#x0D;&#x0A;0"
          checkbox_title="체크박스"
          checkbox_truevalue="1"
          checkbox_falsevalue="0"/>
  </column>

</grid>
```

**Checkbox Properties**:
- Grid level: `use_checkrow="1"` - Enable check row
- Column level: `input_type="1"` - Checkbox cell type
- `checkbox_title` - Checkbox label
- `checkbox_truevalue` - Value when checked
- `checkbox_falsevalue` - Value when unchecked
- `prefilldata` - Initial checkbox states (1=checked, 0=unchecked)

### Grid Input Types

```xml
<column>
  <header title="Combobox"/>
  <data input_type="3"
        combobox_data="del=':' pos='0' style='1' content='0:Option1&#x0D;&#x0A;1:Option2&#x0D;&#x0A;2:Option3'"
        combobox_editable="1"
        combobox_buttonsize="15"/>
</column>

<column>
  <header title="Spinner"/>
  <data input_type="5"
        spinnumber_editable="1"
        spinnumber_showbutton="1"
        spinnumber_min="0"
        spinnumber_max="100"/>
</column>

<column>
  <header title="Calendar"/>
  <data input_type="4"
        calendar_editable="1"/>
</column>
```

**Input Type Values**:
- `input_type="1"` - Checkbox
- `input_type="3"` - Combobox
- `input_type="4"` - Calendar/DatePicker
- `input_type="5"` - Spinner/Numeric

---

## BUTTON - Button Component

### Basic Button

```xml
<pushbutton control_id="978" name="btn_search"
            x="6" y="348" right="716" bottom="408"
            width="90" height="24"
            text="검색"
            font="맑은 고딕,9,0,0,0,0"
            hover_font="맑은 고딕,9,0,0,0,0"
            click_setfocus="0"
            version="1.1"/>
```

### Toggle Button

```xml
<pushbutton control_id="100" name="btn_toggle"
            x="100" y="100"
            width="90" height="24"
            text="토글"
            button_type="2"
            push_state="false"/>
```

**Button Properties**:
- `text` - Button label
- `button_type` - "0" (normal), "2" (toggle)
- `push_state` - true/false (initial pressed state for toggle)
- `click_setfocus` - "0" (no focus), "1" (set focus on click)
- `font` - Font specification: "Family,Size,Bold,Italic,Underline,Strikeout"

---

## FIELD - Input Fields

### Field Types

```xml
<!-- Numeric Field -->
<numericex_field control_id="1" name="field_number"
                 x="10" y="10" width="150" height="24"
                 max_length="10"
                 default_value=""
                 font="맑은 고딕,9,0,0,0,0"/>

<!-- Normal Text Field -->
<normal_field control_id="2" name="field_text"
              x="10" y="40" width="150" height="24"
              max_length="50"
              default_value=""
              font="맑은 고딕,9,0,0,0,0"/>

<!-- Korean Input Field -->
<hangul_field control_id="3" name="field_korean"
              x="10" y="70" width="150" height="24"
              max_length="50"
              default_value=""
              font="맑은 고딕,9,0,0,0,0"/>

<!-- Password Field -->
<password_field control_id="4" name="field_password"
                x="10" y="100" width="150" height="24"
                max_length="20"
                default_value=""
                font="맑은 고딕,9,0,0,0,0"/>
```

**Field Types**:
- `numericex_field` - Numeric input (numbers, decimals, minus only)
- `normal_field` - General text (excludes Korean characters)
- `hangul_field` - Korean character input (all characters)
- `password_field` - Masked input

**Common Properties**:
- `max_length` - Maximum character limit
- `default_value` - Initial value
- `font` - Font specification
- `text_margin` - Internal text spacing

**Events**:
- `on_prekeydown` - Before key processing
- `on_keydown` - During key processing

---

## COMBOBOX - Dropdown Selection

### Basic Combobox with Static Data

```xml
<combobox control_id="104" name="cbo_country"
          x="168" y="612" width="138" height="24"
          default_value="0"
          combobox_data="del=':' pos='0' style='1' content='0:대한민국&#x0D;&#x0A;1:미국&#x0D;&#x0A;2:중국&#x0D;&#x0A;3:일본&#x0D;&#x0A;4:영국'"
          picklist_font="맑은 고딕,9,0,0,0,0"
          picklist_selstyle="1"
          version="1.1"/>
```

### Combobox with Dataset Binding

```xml
<xlinkdataset id="DS_PICKLIST" desc=""
  columns="CODE:&quot;CODE&quot;::&quot;0&#x0A;1&#x0A;2&#x0A;&quot;:&quot;&quot;;
           COMMENT:&quot;COMMENT&quot;::&quot;대한민국&#x0A;미국&#x0A;중국&#x0A;&quot;:&quot;&quot;;
           HIDDEN:&quot;HIDDEN&quot;::&quot;KOR&#x0A;USA&#x0A;CHN&#x0A;&quot;:&quot;&quot;"/>

<combobox control_id="105" name="cbo_dataset"
          x="168" y="650" width="138" height="24"
          link_data="DS_PICKLIST"
          default_value="0"
          picklist_font="맑은 고딕,9,0,0,0,0"
          picklist_viewstyle="0"
          picklist_selstyle="1"/>
```

**Combobox Properties**:
- `combobox_data` - Static data format: `del=':' pos='0' style='1' content='code:display&#x0D;&#x0A;...'`
- `link_data` - Dataset binding
- `default_value` - Initial selection (index or value)
- `default_value_type` - "0" (index), "1" (code), "2" (comment)
- `editable` - "0" (dropdown only), "1" (allow keyboard input)
- `auto_skip` - Auto-focus next control after selection
- `picklist_viewstyle` - "0" (code only), "2" (comment only)
- `picklist_selstyle` - "1" (code+comment), "2" (comment only)

**APIs**:
- `getselectedcode()` - Get selected code
- `setselectedcode()` - Set selection by index/code/comment

---

## PANEL - Container Component

### Basic Panel

```xml
<panel control_id="0" name="pnl_main"
       x="6" y="312" width="400" height="300"
       back_color="00DCB767"
       border="0"
       hidden="0"
       vertscrollbar_style="1"
       horzscrollbar_style="0">

  <!-- Child components -->
  <pushbutton control_id="1" name="btn_1" .../>
  <normal_field control_id="2" name="field_1" .../>

</panel>
```

**Panel Properties**:
- `x`, `y`, `right`, `bottom` - Positioning
- `width`, `height` - Dimensions
- `back_color` - Background color (Hex: RRGGBBAA)
- `border` - Border style ("0" = none)
- `hidden` - Visibility ("0" = visible, "1" = hidden)
- `vertscrollbar_style` - Vertical scrollbar
- `horzscrollbar_style` - Horizontal scrollbar

### Nested Panels with Tab Order

```xml
<panel control_id="0" name="pnl_container" x="10" y="10" width="500" height="400">

  <tab_order order_option="0" order_info="83,85"/>

  <panel control_id="83" name="pnl_section1" x="10" y="10" width="480" height="180">
    <text control_id="84" text="Section 1" .../>
  </panel>

  <panel control_id="85" name="pnl_section2" x="10" y="200" width="480" height="180">
    <text control_id="86" text="Section 2" .../>
  </panel>

</panel>
```

---

## TAB - Tab Navigation

### Tab Control with Items

```xml
<tab control_id="1" name="tab_main"
     x="12" y="768" right="8" bottom="5"
     width="792" height="190"
     tabitem_height="30"
     tabitem_font="맑은 고딕,9,0,0,0,0"
     tabitem_selfont="맑은 고딕,9,0,0,0,0"
     tabitem_xbutton="1"
     tabitem_lockbutton="1"
     tabitem_lockbutton_action="1"
     fixed_lockitem_maxcount="3"
     tabitem_moveable="1"
     titlebar_font="맑은 고딕,9,1,0,0,0"
     tabitem_closeallbtnhidden="0">

  <tab_item title="Tab 1" width="150" height="24" panel_color="00FFFFFF">
    <!-- Tab 1 content -->
    <text control_id="101" text="Content for Tab 1" .../>
  </tab_item>

  <tab_item title="Tab 2" width="150" height="24" panel_color="00FFFFFF">
    <!-- Tab 2 content -->
    <text control_id="102" text="Content for Tab 2" .../>
  </tab_item>

</tab>
```

**Tab Properties**:
- `tabitem_height` - Tab item height
- `tabitem_xbutton` - Show close button ("1" = yes)
- `tabitem_lockbutton` - Show lock button ("1" = yes)
- `tabitem_moveable` - Allow drag-drop reorder ("1" = yes)
- `fixed_lockitem_maxcount` - Maximum locked items
- `tabitem_closeallbtnhidden` - Show close-all button ("0" = show)

**Tab Item Properties**:
- `title` - Tab label
- `width`, `height` - Tab item dimensions
- `panel_color` - Content panel background color

---

## DIV - HTML Container

### Basic Div

```xml
<div control_id="2301" name="div_content"
     x="6" y="582" right="716" bottom="66"
     width="400" height="200"
     text="<div>DIV OBJECT</div>"
     display_type="block"
     padding="10"
     content_overflow="auto"
     custom_class="my-custom-class"
     use_select="1"/>
```

**Div Properties**:
- `text` - HTML content
- `display_type` - CSS display behavior
- `padding` - Internal spacing
- `content_overflow` - Overflow CSS styling
- `custom_class` - CSS class names
- `use_select` - Enable/disable text selection

**APIs**:
- `getdom()` - Returns native HTML element
- `getjdom()` - Returns jQuery-wrapped object
- `setinnerhtml()` - Sets internal HTML content

---

## COMPLETE SCREEN EXAMPLE

### List Screen with Search

```xml
<?xml version="1.0" encoding="UTF-8"?>
<screen id="SCREEN_MEMBER_LIST" width="800" height="600" script_language="Java">

  <!-- Datasets -->
  <xdataset id="ds_search" desc="검색조건">
    <!-- Populated programmatically -->
  </xdataset>

  <xdataset id="ds_list" desc="회원목록">
    <!-- Populated from server -->
  </xdataset>

  <!-- Search Panel -->
  <panel control_id="1" name="pnl_search"
         x="10" y="10" width="780" height="60"
         back_color="00F0F0F0" border="1">

    <text control_id="2" name="txt_name_label"
          x="10" y="20" width="80" height="24"
          text="회원명:" font="맑은 고딕,9,0,0,0,0"/>

    <normal_field control_id="3" name="field_search_name"
                  x="100" y="20" width="200" height="24"
                  max_length="50" font="맑은 고딕,9,0,0,0,0"/>

    <pushbutton control_id="4" name="btn_search"
                x="320" y="18" width="80" height="28"
                text="검색" font="맑은 고딕,9,0,0,0,0"/>
  </panel>

  <!-- Grid Panel -->
  <panel control_id="10" name="pnl_grid"
         x="10" y="80" width="780" height="450">

    <grid control_id="11" name="grid_list"
          x="0" y="0" width="780" height="450"
          link_data="ds_list"
          linenumber_show="1"
          use_checkrow="1"
          on_itemdblclick="eventfunc:grid_list_on_itemdblclick(objInst, nRow, nColumn, buttonClick, imageIndex)">

      <column>
        <header title="회원ID"/>
        <data name="MEMBER_ID"
              link_data="ds_list:MEMBER_ID"
              width="100"
              text_horzalign="1"
              data_type="2"/>
      </column>

      <column>
        <header title="회원명"/>
        <data name="MEMBER_NAME"
              link_data="ds_list:MEMBER_NAME"
              width="150"
              text_horzalign="1"
              data_type="2"/>
      </column>

      <column>
        <header title="이메일"/>
        <data name="EMAIL"
              link_data="ds_list:EMAIL"
              width="200"
              text_horzalign="1"
              data_type="2"/>
      </column>

      <column>
        <header title="등록일"/>
        <data name="REG_DATE"
              link_data="ds_list:REG_DATE"
              width="120"
              text_horzalign="1"
              data_type="2"/>
      </column>

    </grid>
  </panel>

  <!-- Button Panel -->
  <panel control_id="20" name="pnl_buttons"
         x="10" y="540" width="780" height="40">

    <pushbutton control_id="21" name="btn_add"
                x="10" y="8" width="80" height="28"
                text="신규" font="맑은 고딕,9,0,0,0,0"/>

    <pushbutton control_id="22" name="btn_delete"
                x="100" y="8" width="80" height="28"
                text="삭제" font="맑은 고딕,9,0,0,0,0"/>
  </panel>

</screen>
```

---

## NAMING CONVENTIONS (From Patterns)

### Component IDs

**Datasets**: `ds_` prefix
- `ds_list` - List data
- `ds_search` - Search criteria
- `ds_detail` - Detail data
- `ds_result` - Query results

**Grids**: `grid_` or `grd` prefix
- `grid_list` - Main list grid
- `grdList` - Alternative naming
- `grid_detail` - Detail grid

**Buttons**: `btn_` prefix
- `btn_search` - Search button
- `btn_save` - Save button
- `btn_delete` - Delete button
- `btn_add` - Add button

**Fields**: `field_` prefix
- `field_search_name` - Search name field
- `field_member_id` - Member ID field

**Panels**: `pnl_` prefix
- `pnl_search` - Search panel
- `pnl_grid` - Grid container panel
- `pnl_buttons` - Button panel

**Comboboxes**: `cbo_` prefix
- `cbo_country` - Country combobox
- `cbo_status` - Status combobox

**Tabs**: `tab_` prefix
- `tab_main` - Main tab control

**Divs**: `div_` prefix
- `div_content` - Content div
- `div_basic` - Basic div

---

## DATA TYPES

**Grid/Field Data Types**:
- `data_type="0"` - Numeric
- `data_type="2"` - String/Text
- `data_type="3"` - Date
- `data_type="4"` - DateTime

**Number Patterns**:
- `pattern="#,##0"` - Thousand separator, no decimals
- `pattern="#,##0.00"` - Thousand separator, 2 decimals
- `pattern="-ZZ,ZZZ,ZZZ,ZZZ,ZZ9"` - Negative numbers with commas

---

## TEXT ALIGNMENT

**text_horzalign Values**:
- `text_horzalign="0"` - Left align
- `text_horzalign="1"` - Center align
- `text_horzalign="2"` - Right align

---

## SPECIAL CHARACTERS

**HTML Entities**:
- `&#x0A;` - Line feed (LF) - for row separation in dataset
- `&#x0D;&#x0A;` - Carriage return + Line feed (CRLF) - for row separation
- `&quot;` - Quotation mark
- `&lt;` - Less than
- `&gt;` - Greater than
- `&amp;` - Ampersand

---

## VERSION NOTES

- **Source**: xFrame5 TechNet Template Library
- **Last Updated**: 2025-12-28
- **Components Covered**: XDataSet, Grid, Button, Field, Combobox, Panel, Tab, Div
- **Template Count**: 67 component types available
- **Quality**: Actual production templates from vendor
