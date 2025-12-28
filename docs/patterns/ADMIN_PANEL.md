# Admin Panel Pattern (HTMX)

**Purpose**: Document the HTMX-based admin panel architecture for managing prompt templates, company rules, and audit logs.

---

## Technology Decision

### Chosen Stack
- **HTMX**: For dynamic interactions without JavaScript framework
- **Tera**: Server-side templating (built into Loco.rs)
- **Tailwind CSS**: Utility-first CSS styling
- **Loco.rs Views**: Served from agent server (no separate frontend)

### Why HTMX over React/Vue?

| Factor | HTMX | React/Vue |
|--------|------|-----------|
| Deployment | Single server | Separate build/deploy |
| Complexity | Low (HTML attributes) | High (JS framework) |
| CRUD Operations | Ideal fit | Overkill |
| Learning Curve | Minimal | Moderate |
| Primary UI | Eclipse plugin | Admin panel (secondary) |

**Decision**: HTMX is the right choice for a CRUD-focused admin panel that's secondary to the Eclipse plugin.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Admin Panel                          │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Templates   │  │   Rules      │  │   Logs       │  │
│  │   (CRUD)     │  │   (CRUD)     │  │   (View)     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │              HTMX + Tera Templates               │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                               │
│  ┌──────────────────────▼──────────────────────────┐  │
│  │            Loco.rs View Controllers              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## HTMX Patterns

### Pattern 1: Main Page with Search

```html
<!-- main.html -->
<div id="content-body" class="w-full h-screen">
    <h1 class="text-3xl pb-6">Prompt Templates</h1>

    <!-- Search Form (triggers on submit and page load) -->
    <form id="search-form"
          hx-get="/admin/templates/list"
          hx-target="#search-result"
          hx-trigger="submit, load"
          hx-ext="json-enc"
          hx-swap="outerHTML">
        <input name="keyword" type="text" placeholder="Search...">
        <button type="submit">Search</button>
    </form>

    <!-- List Container (updated by HTMX) -->
    {% include "admin/template/list.html" %}

    <!-- Editor Modal Container -->
    <div id="item-editor-container"></div>
</div>
```

### Pattern 2: Paginated List

```html
<!-- list.html -->
<div id="search-result">
    <table class="min-w-full">
        <thead>
            <tr>
                <th>Name</th>
                <th>Product</th>
                <th>Screen Type</th>
                <th>Active</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>
            {% for item in items %}
            {% include 'admin/template/row.html' %}
            {% endfor %}
        </tbody>
    </table>

    <!-- Pagination with delayed input trigger -->
    {% if page and total_pages %}
    <div class="pagination">
        <input type="number"
               value="{{page}}"
               min="1"
               max="{{total_pages}}"
               name="page"
               form="search-form"
               hx-get="/admin/templates/list"
               hx-target="#search-result"
               hx-swap="outerHTML"
               hx-trigger="input changed delay:0.5s" />
        <span>/ {{total_pages}}</span>
    </div>
    {% endif %}
</div>
```

### Pattern 3: Table Row

```html
<!-- row.html -->
<tr class="tr_{{item.id}}">
    <td>{{item.name}}</td>
    <td>{{item.product}}</td>
    <td>{{item.screen_type | default(value="-") }}</td>
    <td>{{item.is_active}}</td>
    <td>
        <button hx-get="/admin/templates/{{item.id}}/edit"
                hx-target="#item-editor-container"
                hx-swap="innerHTML">
            Edit
        </button>
        <button hx-delete="/admin/templates/{{item.id}}"
                hx-target=".tr_{{item.id}}"
                hx-swap="outerHTML"
                hx-confirm="Delete this template?">
            Delete
        </button>
    </td>
</tr>
```

### Pattern 4: Edit Modal

```html
<!-- edit.html -->
<div id="item-editor">
    <div class="modal-overlay">
        <div class="modal-content">
            <h2>Edit Template: {{item.name}}</h2>

            <form id="editor-form"
                  hx-patch="/admin/templates/{{item.id}}"
                  hx-ext="json-enc"
                  hx-target="#search-result table tbody .tr_{{item.id}}"
                  hx-swap="outerHTML">

                <label>Name</label>
                <input type="text" name="name" value="{{item.name}}" required />

                <label>Product</label>
                <input type="text" name="product" value="{{item.product}}" required />

                <label>Screen Type</label>
                <select name="screen_type">
                    <option value="list" {% if item.screen_type == "list" %}selected{% endif %}>List</option>
                    <option value="detail" {% if item.screen_type == "detail" %}selected{% endif %}>Detail</option>
                    <option value="popup" {% if item.screen_type == "popup" %}selected{% endif %}>Popup</option>
                </select>

                <label>System Prompt</label>
                <textarea name="system_prompt" rows="10">{{item.system_prompt}}</textarea>

                <label>User Prompt Template</label>
                <textarea name="user_prompt_template" rows="10">{{item.user_prompt_template}}</textarea>

                <label>
                    <input type="checkbox" name="is_active" {% if item.is_active %}checked{% endif %} />
                    Active
                </label>
            </form>

            <div class="actions">
                <button form="editor-form" type="submit">Save</button>
                <button hx-post="/empty"
                        hx-target="#item-editor-container"
                        hx-swap="innerHTML">
                    Close
                </button>
            </div>
        </div>
    </div>
</div>
```

### Pattern 5: Create Form

```html
<!-- create.html -->
<div id="item-editor">
    <div class="modal-overlay">
        <div class="modal-content">
            <h2>New Template</h2>

            <form id="editor-form"
                  hx-post="/admin/templates"
                  hx-ext="json-enc"
                  hx-target="#search-result"
                  hx-swap="outerHTML">
                <!-- Same fields as edit, but without values -->
            </form>

            <div class="actions">
                <button form="editor-form" type="submit">Create</button>
                <button hx-post="/empty"
                        hx-target="#item-editor-container"
                        hx-swap="innerHTML">
                    Cancel
                </button>
            </div>
        </div>
    </div>
</div>
```

---

## Loco.rs View Controllers

### Controller Structure

```rust
// backend/src/controllers/admin/prompt_templates.rs
use loco_rs::prelude::*;
use tera::Context;

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(main_page))
        .add("/list", get(list))
        .add("/new", get(new_form))
        .add("/", post(create))
        .add("/:id", get(show))
        .add("/:id/edit", get(edit_form))
        .add("/:id", patch(update))
        .add("/:id", delete(destroy))
}

async fn main_page(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Render main page (list will be loaded via HTMX)
    format::render().view(&v, "admin/prompt_template/main.html", data!({}))
}

async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let per_page = 20;

    let (items, total) = PromptTemplateService::paginate(
        &ctx.db,
        params.keyword.as_deref(),
        page,
        per_page
    ).await?;

    let total_pages = (total + per_page - 1) / per_page;

    format::render().view(&v, "admin/prompt_template/list.html", data!({
        "items": items,
        "page": page,
        "total_pages": total_pages,
    }))
}

async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let item = PromptTemplateService::find_by_id(&ctx.db, id).await?;

    format::render().view(&v, "admin/prompt_template/edit.html", data!({
        "item": item,
    }))
}

async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    let item = PromptTemplateService::update(&ctx.db, id, params).await?;

    // Return updated row for inline replacement
    format::render().view(&v, "admin/prompt_template/row.html", data!({
        "item": item,
    }))
}
```

### Empty Response Endpoint

```rust
// For closing modals
async fn empty() -> impl IntoResponse {
    ""
}
```

---

## Directory Structure

```
backend/
├── assets/
│   ├── static/
│   │   ├── css/
│   │   │   └── admin.css       # Tailwind-generated CSS
│   │   └── js/
│   │       └── htmx.min.js     # HTMX library
│   └── views/
│       ├── admin/
│       │   ├── layout.html     # Base layout
│       │   ├── prompt_template/
│       │   │   ├── main.html
│       │   │   ├── list.html
│       │   │   ├── row.html
│       │   │   ├── create.html
│       │   │   ├── edit.html
│       │   │   └── show.html
│       │   ├── company_rule/
│       │   │   └── ...
│       │   ├── generation_log/
│       │   │   └── ...
│       │   └── llm_config/
│       │       └── ...
│       └── shared/
│           ├── pagination.html
│           └── modal.html
└── src/
    └── controllers/
        └── admin/
            ├── mod.rs
            ├── prompt_templates.rs
            ├── company_rules.rs
            ├── generation_logs.rs
            └── llm_configs.rs
```

---

## Key HTMX Attributes Reference

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `hx-get` | GET request on trigger | `hx-get="/admin/items/list"` |
| `hx-post` | POST request | `hx-post="/admin/items"` |
| `hx-patch` | PATCH request (update) | `hx-patch="/admin/items/1"` |
| `hx-delete` | DELETE request | `hx-delete="/admin/items/1"` |
| `hx-target` | Where to put response | `hx-target="#search-result"` |
| `hx-swap` | How to insert response | `hx-swap="outerHTML"` |
| `hx-trigger` | When to send request | `hx-trigger="submit, load"` |
| `hx-ext` | Extensions | `hx-ext="json-enc"` |
| `hx-confirm` | Confirmation dialog | `hx-confirm="Are you sure?"` |
| `hx-indicator` | Loading indicator | `hx-indicator=".spinner"` |

---

## Security Considerations

### Authentication
- All `/admin/*` routes require authentication
- Use Loco.rs middleware for session validation
- Redirect to login on unauthorized access

### Authorization
- Admin role required for template management
- LLM config management restricted to super-admin
- Audit logs are read-only

### CSRF Protection
- Include CSRF token in forms
- Validate token on all mutations

```html
<form hx-post="/admin/templates">
    <input type="hidden" name="_csrf" value="{{csrf_token}}" />
    <!-- fields -->
</form>
```

---

## Reference Implementation

The HTMX patterns in this document are based on the **yatclub** project:
- Repository: `/Users/kibong/development/yatclub/`
- Key templates: `assets/views/admin/member/`

---

## Responsive Layout (yatclub Pattern + shadcn Styling)

### Base Layout Template

Reference: `yatclub/assets/views/home/adminhome.html` for HTMX structure, `starshare-app` for shadcn styling.

```html
<!-- base-admin.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{% block title %}{% endblock title %} - xFrame5 Admin</title>

    <!-- HTMX -->
    <script src="https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js"></script>
    <script src="https://unpkg.com/htmx-ext-json-enc@2.0.1/json-enc.js"></script>

    <!-- Tailwind with shadcn preset (production: use compiled CSS) -->
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    colors: {
                        background: 'hsl(var(--background))',
                        foreground: 'hsl(var(--foreground))',
                        card: { DEFAULT: 'hsl(var(--card))', foreground: 'hsl(var(--card-foreground))' },
                        primary: { DEFAULT: 'hsl(var(--primary))', foreground: 'hsl(var(--primary-foreground))' },
                        secondary: { DEFAULT: 'hsl(var(--secondary))', foreground: 'hsl(var(--secondary-foreground))' },
                        muted: { DEFAULT: 'hsl(var(--muted))', foreground: 'hsl(var(--muted-foreground))' },
                        accent: { DEFAULT: 'hsl(var(--accent))', foreground: 'hsl(var(--accent-foreground))' },
                        destructive: { DEFAULT: 'hsl(var(--destructive))', foreground: 'hsl(var(--destructive-foreground))' },
                        border: 'hsl(var(--border))',
                        input: 'hsl(var(--input))',
                        ring: 'hsl(var(--ring))',
                        sidebar: {
                            DEFAULT: 'hsl(var(--sidebar))',
                            foreground: 'hsl(var(--sidebar-foreground))',
                            accent: 'hsl(var(--sidebar-accent))',
                            'accent-foreground': 'hsl(var(--sidebar-accent-foreground))',
                            border: 'hsl(var(--sidebar-border))',
                        },
                    },
                    borderRadius: {
                        lg: 'var(--radius)',
                        md: 'calc(var(--radius) - 2px)',
                        sm: 'calc(var(--radius) - 4px)',
                    },
                }
            }
        }
    </script>
    <style>
        :root {
            --radius: 0.625rem;
            --background: 0 0% 100%;
            --foreground: 224 71% 4%;
            --card: 0 0% 100%;
            --card-foreground: 224 71% 4%;
            --primary: 220 14% 10%;
            --primary-foreground: 210 20% 98%;
            --secondary: 220 14% 96%;
            --secondary-foreground: 220 14% 10%;
            --muted: 220 14% 96%;
            --muted-foreground: 220 9% 46%;
            --accent: 220 14% 96%;
            --accent-foreground: 220 14% 10%;
            --destructive: 0 84% 60%;
            --destructive-foreground: 210 20% 98%;
            --border: 220 13% 91%;
            --input: 220 13% 91%;
            --ring: 224 71% 4%;
            --sidebar: 0 0% 98%;
            --sidebar-foreground: 224 71% 4%;
            --sidebar-accent: 220 14% 96%;
            --sidebar-accent-foreground: 220 14% 10%;
            --sidebar-border: 220 13% 91%;
        }
        .dark {
            --background: 224 71% 4%;
            --foreground: 210 20% 98%;
            --card: 224 71% 6%;
            --card-foreground: 210 20% 98%;
            --primary: 210 20% 98%;
            --primary-foreground: 224 71% 4%;
            --secondary: 215 28% 17%;
            --secondary-foreground: 210 20% 98%;
            --muted: 215 28% 17%;
            --muted-foreground: 218 11% 65%;
            --accent: 215 28% 17%;
            --accent-foreground: 210 20% 98%;
            --destructive: 0 63% 31%;
            --destructive-foreground: 210 20% 98%;
            --border: 215 28% 17%;
            --input: 215 28% 17%;
            --ring: 216 12% 84%;
            --sidebar: 224 71% 4%;
            --sidebar-foreground: 210 20% 98%;
            --sidebar-accent: 215 28% 17%;
            --sidebar-accent-foreground: 210 20% 98%;
            --sidebar-border: 215 28% 17%;
        }
        * { border-color: hsl(var(--border)); }
        body { background-color: hsl(var(--background)); color: hsl(var(--foreground)); }
    </style>

    {% block head %}{% endblock head %}
</head>
<body class="min-h-screen antialiased">
    {% block content %}{% endblock content %}

    {% block js %}{% endblock js %}
</body>
</html>
```

### Responsive Sidebar Layout

```html
<!-- layout-admin.html -->
{% extends "base-admin.html" %}
{% block content %}
<div class="flex min-h-screen">
    <!-- Desktop Sidebar (hidden on mobile) -->
    <aside class="hidden md:flex md:w-64 md:flex-col md:fixed md:inset-y-0 z-50">
        <div class="flex flex-col flex-grow bg-sidebar border-r border-sidebar-border overflow-y-auto">
            <!-- Logo/Brand -->
            <div class="flex items-center h-16 px-4 border-b border-sidebar-border">
                <span class="text-xl font-semibold text-sidebar-foreground">xFrame5 Admin</span>
            </div>

            <!-- Navigation -->
            <nav class="flex-1 px-2 py-4 space-y-1">
                {% include "admin/partials/sidebar-nav.html" %}
            </nav>

            <!-- User Menu (Footer) -->
            <div class="flex-shrink-0 p-4 border-t border-sidebar-border">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center">
                        <span class="text-sm font-medium">U</span>
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-sm font-medium text-sidebar-foreground truncate">{{user.name}}</p>
                        <p class="text-xs text-muted-foreground truncate">{{user.email}}</p>
                    </div>
                </div>
            </div>
        </div>
    </aside>

    <!-- Mobile Header (hidden on desktop) -->
    <header class="md:hidden fixed inset-x-0 top-0 z-50 bg-background border-b border-border">
        <div class="flex items-center justify-between h-14 px-4">
            <span class="text-lg font-semibold">xFrame5 Admin</span>
            <button type="button" onclick="toggleMobileMenu()"
                class="inline-flex items-center justify-center rounded-md p-2 text-foreground hover:bg-accent">
                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                </svg>
            </button>
        </div>
    </header>

    <!-- Mobile Slide-over Menu -->
    <div id="mobile-menu" class="hidden md:hidden fixed inset-0 z-50">
        <div class="fixed inset-0 bg-black/50" onclick="toggleMobileMenu()"></div>
        <div class="fixed inset-y-0 left-0 w-full max-w-xs bg-sidebar">
            <div class="flex items-center justify-between h-14 px-4 border-b border-sidebar-border">
                <span class="text-lg font-semibold text-sidebar-foreground">xFrame5 Admin</span>
                <button type="button" onclick="toggleMobileMenu()"
                    class="rounded-md p-2 text-sidebar-foreground hover:bg-sidebar-accent">
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
            <nav class="px-2 py-4 space-y-1">
                {% include "admin/partials/sidebar-nav-mobile.html" %}
            </nav>
        </div>
    </div>

    <!-- Main Content Area -->
    <main class="flex-1 md:ml-64">
        <div class="pt-14 md:pt-0">
            <div id="content-body" class="p-4 md:p-6">
                {% block main %}{% endblock main %}
            </div>
        </div>
    </main>
</div>

<script>
function toggleMobileMenu() {
    const menu = document.getElementById('mobile-menu');
    menu.classList.toggle('hidden');
}
</script>
{% endblock content %}
```

### Sidebar Navigation (HTMX Pattern)

```html
<!-- admin/partials/sidebar-nav.html -->
<!-- Desktop: buttons with HTMX, targeting #content-body -->
<button hx-get="/admin/templates" hx-target="#content-body" hx-swap="innerHTML"
    class="group flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-md
           text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
           {% if current_page == 'templates' %}bg-sidebar-accent text-sidebar-accent-foreground{% endif %}">
    <svg class="h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
    </svg>
    Prompt Templates
</button>

<button hx-get="/admin/rules" hx-target="#content-body" hx-swap="innerHTML"
    class="group flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-md
           text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
           {% if current_page == 'rules' %}bg-sidebar-accent text-sidebar-accent-foreground{% endif %}">
    <svg class="h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" />
    </svg>
    Company Rules
</button>

<button hx-get="/admin/logs" hx-target="#content-body" hx-swap="innerHTML"
    class="group flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-md
           text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
           {% if current_page == 'logs' %}bg-sidebar-accent text-sidebar-accent-foreground{% endif %}">
    <svg class="h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
    </svg>
    Generation Logs
</button>

<button hx-get="/admin/llm-config" hx-target="#content-body" hx-swap="innerHTML"
    class="group flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-md
           text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
           {% if current_page == 'llm-config' %}bg-sidebar-accent text-sidebar-accent-foreground{% endif %}">
    <svg class="h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 3.104v5.714a2.25 2.25 0 01-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 014.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0112 15a9.065 9.065 0 00-6.23.693L5 15.5m14.8-.2a2.25 2.25 0 00.681-1.591V8.25a2.25 2.25 0 00-.659-1.591L15.75 3M5 14.5l-.08.02A2.25 2.25 0 003 16.72v.03c0 1.181.91 2.166 2.086 2.243l.924.074a24.427 24.427 0 0011.98 0l.924-.074A2.25 2.25 0 0021 16.75v-.03a2.25 2.25 0 00-1.92-2.201L19 14.5" />
    </svg>
    LLM Config
</button>
```

### Mobile Navigation (closes menu on click)

```html
<!-- admin/partials/sidebar-nav-mobile.html -->
<!-- Mobile: same as desktop but with onclick="toggleMobileMenu()" -->
<button hx-get="/admin/templates" hx-target="#content-body" hx-swap="innerHTML" onclick="toggleMobileMenu()"
    class="group flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-md
           text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground">
    <svg class="h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
    </svg>
    Prompt Templates
</button>
<!-- ... repeat for other nav items -->
```

---

## shadcn Component Classes Reference

### Button Variants (Tailwind equivalents)

```html
<!-- Default Button -->
<button class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium
               h-9 px-4 py-2 bg-primary text-primary-foreground shadow-sm hover:bg-primary/90
               disabled:pointer-events-none disabled:opacity-50">
    Save Changes
</button>

<!-- Secondary Button -->
<button class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium
               h-9 px-4 py-2 bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80">
    Cancel
</button>

<!-- Outline Button -->
<button class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium
               h-9 px-4 py-2 border bg-background shadow-sm hover:bg-accent hover:text-accent-foreground">
    Edit
</button>

<!-- Ghost Button -->
<button class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium
               h-9 px-4 py-2 hover:bg-accent hover:text-accent-foreground">
    View Details
</button>

<!-- Destructive Button -->
<button class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium
               h-9 px-4 py-2 bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90">
    Delete
</button>

<!-- Icon Button -->
<button class="inline-flex items-center justify-center rounded-md text-sm font-medium
               h-9 w-9 hover:bg-accent hover:text-accent-foreground">
    <svg class="h-4 w-4" ...></svg>
</button>

<!-- Small Button -->
<button class="inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md text-sm font-medium
               h-8 px-3 bg-primary text-primary-foreground shadow-sm hover:bg-primary/90">
    Add
</button>
```

### Card Component

```html
<!-- Card -->
<div class="bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm">
    <!-- Card Header -->
    <div class="grid auto-rows-min gap-1.5 px-6">
        <div class="leading-none font-semibold">Prompt Templates</div>
        <div class="text-muted-foreground text-sm">Manage your prompt templates</div>
    </div>

    <!-- Card Content -->
    <div class="px-6">
        <!-- content here -->
    </div>

    <!-- Card Footer (optional) -->
    <div class="flex items-center px-6 border-t pt-6">
        <!-- footer content -->
    </div>
</div>
```

### Input Component

```html
<!-- Input -->
<input type="text" name="name"
    class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm
           placeholder:text-muted-foreground
           focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring
           disabled:cursor-not-allowed disabled:opacity-50" />

<!-- Textarea -->
<textarea name="content" rows="10"
    class="flex min-h-[60px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm
           placeholder:text-muted-foreground
           focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring
           disabled:cursor-not-allowed disabled:opacity-50"></textarea>

<!-- Select -->
<select name="screen_type"
    class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm
           focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
    <option value="">Select type...</option>
    <option value="list">List</option>
    <option value="detail">Detail</option>
</select>
```

### Table Component

```html
<div class="relative w-full overflow-auto">
    <table class="w-full caption-bottom text-sm">
        <thead class="[&_tr]:border-b">
            <tr class="border-b transition-colors hover:bg-muted/50">
                <th class="h-10 px-2 text-left align-middle font-medium text-muted-foreground">Name</th>
                <th class="h-10 px-2 text-left align-middle font-medium text-muted-foreground">Product</th>
                <th class="h-10 px-2 text-left align-middle font-medium text-muted-foreground">Status</th>
                <th class="h-10 px-2 text-right align-middle font-medium text-muted-foreground">Actions</th>
            </tr>
        </thead>
        <tbody class="[&_tr:last-child]:border-0">
            {% for item in items %}
            <tr class="tr_{{item.id}} border-b transition-colors hover:bg-muted/50">
                <td class="p-2 align-middle">{{item.name}}</td>
                <td class="p-2 align-middle">{{item.product}}</td>
                <td class="p-2 align-middle">
                    {% if item.is_active %}
                    <span class="inline-flex items-center rounded-md bg-green-50 px-2 py-1 text-xs font-medium text-green-700 ring-1 ring-inset ring-green-600/20">Active</span>
                    {% else %}
                    <span class="inline-flex items-center rounded-md bg-gray-50 px-2 py-1 text-xs font-medium text-gray-600 ring-1 ring-inset ring-gray-500/10">Inactive</span>
                    {% endif %}
                </td>
                <td class="p-2 align-middle text-right">
                    <button hx-get="/admin/templates/{{item.id}}/edit" hx-target="#modal-container" hx-swap="innerHTML"
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium h-8 px-3 hover:bg-accent hover:text-accent-foreground">
                        Edit
                    </button>
                </td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
</div>
```

### Badge Component

```html
<!-- Default Badge -->
<span class="inline-flex items-center rounded-md bg-primary/10 px-2 py-1 text-xs font-medium text-primary ring-1 ring-inset ring-primary/20">
    Default
</span>

<!-- Success Badge -->
<span class="inline-flex items-center rounded-md bg-green-50 px-2 py-1 text-xs font-medium text-green-700 ring-1 ring-inset ring-green-600/20 dark:bg-green-500/10 dark:text-green-400 dark:ring-green-500/20">
    Active
</span>

<!-- Destructive Badge -->
<span class="inline-flex items-center rounded-md bg-red-50 px-2 py-1 text-xs font-medium text-red-700 ring-1 ring-inset ring-red-600/10 dark:bg-red-500/10 dark:text-red-400 dark:ring-red-500/20">
    Error
</span>

<!-- Secondary Badge -->
<span class="inline-flex items-center rounded-md bg-secondary px-2 py-1 text-xs font-medium text-secondary-foreground">
    Inactive
</span>
```

### Modal Dialog

```html
<!-- Modal Container (in main layout) -->
<div id="modal-container"></div>

<!-- Modal Content (loaded via HTMX) -->
<div id="modal-overlay" class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Backdrop -->
    <div class="fixed inset-0 bg-black/50" hx-post="/empty" hx-target="#modal-container" hx-swap="innerHTML"></div>

    <!-- Modal -->
    <div class="relative bg-background rounded-lg shadow-lg border w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto">
        <!-- Header -->
        <div class="flex items-center justify-between p-4 border-b">
            <h2 class="text-lg font-semibold">Edit Template</h2>
            <button hx-post="/empty" hx-target="#modal-container" hx-swap="innerHTML"
                class="rounded-md p-1 hover:bg-accent">
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>

        <!-- Body -->
        <form id="edit-form" hx-patch="/admin/templates/{{item.id}}" hx-ext="json-enc"
              hx-target=".tr_{{item.id}}" hx-swap="outerHTML">
            <div class="p-4 space-y-4">
                <div class="space-y-2">
                    <label class="text-sm font-medium leading-none">Name</label>
                    <input type="text" name="name" value="{{item.name}}"
                        class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm
                               focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
                </div>
                <!-- more fields -->
            </div>
        </form>

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2 p-4 border-t">
            <button hx-post="/empty" hx-target="#modal-container" hx-swap="innerHTML"
                class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2
                       border bg-background shadow-sm hover:bg-accent hover:text-accent-foreground">
                Cancel
            </button>
            <button form="edit-form" type="submit"
                class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2
                       bg-primary text-primary-foreground shadow-sm hover:bg-primary/90">
                Save Changes
            </button>
        </div>
    </div>
</div>
```

---

## Reference Implementation

The HTMX patterns in this document are based on the **yatclub** project:
- Repository: `/Users/kibong/development/yatclub/`
- Key templates: `assets/views/admin/member/`, `assets/views/home/adminhome.html`
- Responsive patterns: `assets/views/partials/header-admin.html`

The shadcn styling is based on **starshare-app**:
- Repository: `/Users/kibong/development/starshare-app/`
- CSS variables: `src/styles/theme.css`
- Component patterns: `src/shared/components/ui/`

---

**Version**: 1.1.0
**Last Updated**: 2025-12-28
