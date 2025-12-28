package com.kosac.ai.codegen.dialogs;

import org.eclipse.jface.dialogs.TitleAreaDialog;
import org.eclipse.swt.SWT;
import org.eclipse.swt.layout.GridData;
import org.eclipse.swt.layout.GridLayout;
import org.eclipse.swt.widgets.Button;
import org.eclipse.swt.widgets.Combo;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.swt.widgets.Control;
import org.eclipse.swt.widgets.Label;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.swt.widgets.Text;

/**
 * Dialog for inputting a single column definition.
 */
public class ColumnInputDialog extends TitleAreaDialog {

    private Text nameText;
    private Combo typeCombo;
    private Button nullableCheck;
    private Button pkCheck;
    private Text commentText;

    private String name;
    private String type;
    private boolean nullable;
    private boolean pk;
    private String comment;

    private static final String[] COMMON_TYPES = {
            "VARCHAR(100)",
            "VARCHAR(255)",
            "VARCHAR(50)",
            "INTEGER",
            "BIGINT",
            "DECIMAL(10,2)",
            "BOOLEAN",
            "DATE",
            "DATETIME",
            "TIMESTAMP",
            "TEXT",
            "CLOB"
    };

    public ColumnInputDialog(Shell parentShell) {
        super(parentShell);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Add Column");
        setMessage("Enter the column details.");
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

        // Column name
        Label nameLabel = new Label(container, SWT.NONE);
        nameLabel.setText("Column Name:");

        nameText = new Text(container, SWT.BORDER);
        nameText.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));

        // Column type
        Label typeLabel = new Label(container, SWT.NONE);
        typeLabel.setText("Column Type:");

        typeCombo = new Combo(container, SWT.BORDER);
        typeCombo.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        typeCombo.setItems(COMMON_TYPES);

        // Nullable
        Label nullableLabel = new Label(container, SWT.NONE);
        nullableLabel.setText("Nullable:");

        nullableCheck = new Button(container, SWT.CHECK);
        nullableCheck.setSelection(true);

        // Primary Key
        Label pkLabel = new Label(container, SWT.NONE);
        pkLabel.setText("Primary Key:");

        pkCheck = new Button(container, SWT.CHECK);

        // Comment
        Label commentLabel = new Label(container, SWT.NONE);
        commentLabel.setText("Comment:");

        commentText = new Text(container, SWT.BORDER);
        commentText.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));

        return area;
    }

    @Override
    protected void okPressed() {
        if (validateInput()) {
            name = nameText.getText().trim();
            type = typeCombo.getText().trim();
            nullable = nullableCheck.getSelection();
            pk = pkCheck.getSelection();
            comment = commentText.getText().trim();
            if (comment.isEmpty()) {
                comment = null;
            }
            super.okPressed();
        }
    }

    private boolean validateInput() {
        if (nameText.getText().trim().isEmpty()) {
            setErrorMessage("Column name is required.");
            return false;
        }

        if (typeCombo.getText().trim().isEmpty()) {
            setErrorMessage("Column type is required.");
            return false;
        }

        setErrorMessage(null);
        return true;
    }

    public String getName() {
        return name;
    }

    public String getType() {
        return type;
    }

    public boolean isNullable() {
        return nullable;
    }

    public boolean isPk() {
        return pk;
    }

    public String getComment() {
        return comment;
    }
}
