package com.softbase.xframe5.codegen.dialogs;

import java.util.ArrayList;
import java.util.List;

import org.eclipse.jface.dialogs.IDialogConstants;
import org.eclipse.jface.dialogs.TitleAreaDialog;
import org.eclipse.swt.SWT;
import org.eclipse.swt.events.SelectionAdapter;
import org.eclipse.swt.events.SelectionEvent;
import org.eclipse.swt.layout.GridData;
import org.eclipse.swt.layout.GridLayout;
import org.eclipse.swt.widgets.Button;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.swt.widgets.Control;
import org.eclipse.swt.widgets.Label;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.swt.widgets.Table;
import org.eclipse.swt.widgets.TableColumn;
import org.eclipse.swt.widgets.TableItem;
import org.eclipse.swt.widgets.Text;

import com.softbase.xframe5.codegen.model.SchemaInput;
import com.softbase.xframe5.codegen.model.SchemaInput.SchemaColumn;

/**
 * Dialog for inputting database schema information.
 */
public class SchemaInputDialog extends TitleAreaDialog {

    private Text tableNameText;
    private Text schemaNameText;
    private Table columnsTable;

    private SchemaInput schemaInput;

    public SchemaInputDialog(Shell parentShell) {
        super(parentShell);
        setShellStyle(getShellStyle() | SWT.RESIZE);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Generate from DB Schema");
        setMessage("Enter the database table schema to generate xFrame5 code.");
    }

    @Override
    protected Control createDialogArea(Composite parent) {
        Composite area = (Composite) super.createDialogArea(parent);

        Composite container = new Composite(area, SWT.NONE);
        container.setLayoutData(new GridData(SWT.FILL, SWT.FILL, true, true));
        GridLayout layout = new GridLayout(2, false);
        layout.marginHeight = 10;
        layout.marginWidth = 10;
        container.setLayout(layout);

        // Table name
        Label tableLabel = new Label(container, SWT.NONE);
        tableLabel.setText("Table Name:");

        tableNameText = new Text(container, SWT.BORDER);
        tableNameText.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        tableNameText.setMessage("e.g., member");

        // Schema name (optional)
        Label schemaLabel = new Label(container, SWT.NONE);
        schemaLabel.setText("Schema (Optional):");

        schemaNameText = new Text(container, SWT.BORDER);
        schemaNameText.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        schemaNameText.setMessage("e.g., public");

        // Columns section
        Label columnsLabel = new Label(container, SWT.NONE);
        columnsLabel.setText("Columns:");
        columnsLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Composite columnsContainer = new Composite(container, SWT.NONE);
        columnsContainer.setLayoutData(new GridData(SWT.FILL, SWT.FILL, true, true));
        columnsContainer.setLayout(new GridLayout(2, false));

        // Columns table
        columnsTable = new Table(columnsContainer, SWT.BORDER | SWT.FULL_SELECTION | SWT.MULTI);
        GridData tableData = new GridData(SWT.FILL, SWT.FILL, true, true);
        tableData.heightHint = 200;
        columnsTable.setLayoutData(tableData);
        columnsTable.setHeaderVisible(true);
        columnsTable.setLinesVisible(true);

        TableColumn nameCol = new TableColumn(columnsTable, SWT.NONE);
        nameCol.setText("Column Name");
        nameCol.setWidth(120);

        TableColumn typeCol = new TableColumn(columnsTable, SWT.NONE);
        typeCol.setText("Type");
        typeCol.setWidth(100);

        TableColumn nullableCol = new TableColumn(columnsTable, SWT.NONE);
        nullableCol.setText("Nullable");
        nullableCol.setWidth(60);

        TableColumn pkCol = new TableColumn(columnsTable, SWT.NONE);
        pkCol.setText("PK");
        pkCol.setWidth(40);

        TableColumn commentCol = new TableColumn(columnsTable, SWT.NONE);
        commentCol.setText("Comment");
        commentCol.setWidth(150);

        // Buttons for column management
        Composite buttonContainer = new Composite(columnsContainer, SWT.NONE);
        buttonContainer.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));
        buttonContainer.setLayout(new GridLayout(1, false));

        Button addButton = new Button(buttonContainer, SWT.PUSH);
        addButton.setText("Add");
        addButton.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, false, false));
        addButton.addSelectionListener(new SelectionAdapter() {
            @Override
            public void widgetSelected(SelectionEvent e) {
                addColumn();
            }
        });

        Button removeButton = new Button(buttonContainer, SWT.PUSH);
        removeButton.setText("Remove");
        removeButton.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, false, false));
        removeButton.addSelectionListener(new SelectionAdapter() {
            @Override
            public void widgetSelected(SelectionEvent e) {
                removeSelectedColumns();
            }
        });

        Button sampleButton = new Button(buttonContainer, SWT.PUSH);
        sampleButton.setText("Sample");
        sampleButton.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, false, false));
        sampleButton.addSelectionListener(new SelectionAdapter() {
            @Override
            public void widgetSelected(SelectionEvent e) {
                loadSampleData();
            }
        });

        return area;
    }

    private void addColumn() {
        ColumnInputDialog dialog = new ColumnInputDialog(getShell());
        if (dialog.open() == IDialogConstants.OK_ID) {
            TableItem item = new TableItem(columnsTable, SWT.NONE);
            item.setText(0, dialog.getName());
            item.setText(1, dialog.getType());
            item.setText(2, dialog.isNullable() ? "Yes" : "No");
            item.setText(3, dialog.isPk() ? "Yes" : "");
            item.setText(4, dialog.getComment() != null ? dialog.getComment() : "");
        }
    }

    private void removeSelectedColumns() {
        int[] indices = columnsTable.getSelectionIndices();
        if (indices.length > 0) {
            columnsTable.remove(indices);
        }
    }

    private void loadSampleData() {
        tableNameText.setText("member");

        columnsTable.removeAll();

        String[][] sampleData = {
                {"id", "INTEGER", "No", "Yes", ""},
                {"name", "VARCHAR(100)", "No", "", "Name"},
                {"email", "VARCHAR(255)", "Yes", "", "Email Address"},
                {"phone", "VARCHAR(20)", "Yes", "", "Phone Number"},
                {"status", "VARCHAR(10)", "No", "", "Status"},
                {"created_at", "DATETIME", "No", "", "Registration Date"},
                {"updated_at", "DATETIME", "Yes", "", "Last Update"}
        };

        for (String[] row : sampleData) {
            TableItem item = new TableItem(columnsTable, SWT.NONE);
            item.setText(row);
        }
    }

    @Override
    protected void okPressed() {
        if (validateInput()) {
            buildSchemaInput();
            super.okPressed();
        }
    }

    private boolean validateInput() {
        String tableName = tableNameText.getText().trim();
        if (tableName.isEmpty()) {
            setErrorMessage("Table name is required.");
            return false;
        }

        if (columnsTable.getItemCount() == 0) {
            setErrorMessage("At least one column is required.");
            return false;
        }

        setErrorMessage(null);
        return true;
    }

    private void buildSchemaInput() {
        schemaInput = new SchemaInput(tableNameText.getText().trim());

        String schema = schemaNameText.getText().trim();
        if (!schema.isEmpty()) {
            schemaInput.setSchema(schema);
        }

        List<String> pks = new ArrayList<>();
        for (TableItem item : columnsTable.getItems()) {
            SchemaColumn col = new SchemaColumn(item.getText(0), item.getText(1));
            col.setNullable("Yes".equals(item.getText(2)));
            col.setPk("Yes".equals(item.getText(3)));
            col.setComment(item.getText(4).isEmpty() ? null : item.getText(4));

            if (col.isPk()) {
                pks.add(col.getName());
            }

            schemaInput.addColumn(col);
        }

        schemaInput.setPrimaryKeys(pks);
    }

    public SchemaInput getSchemaInput() {
        return schemaInput;
    }

    @Override
    protected boolean isResizable() {
        return true;
    }
}
