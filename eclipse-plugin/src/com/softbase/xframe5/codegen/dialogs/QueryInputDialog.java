package com.softbase.xframe5.codegen.dialogs;

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
import org.eclipse.swt.widgets.Text;

import com.softbase.xframe5.codegen.model.QuerySampleInput;

/**
 * Dialog for inputting SQL query sample.
 */
public class QueryInputDialog extends TitleAreaDialog {

    private Text queryText;
    private Text descriptionText;

    private QuerySampleInput queryInput;

    public QueryInputDialog(Shell parentShell) {
        super(parentShell);
        setShellStyle(getShellStyle() | SWT.RESIZE);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Generate from Query Sample");
        setMessage("Enter a SELECT query to generate xFrame5 code based on the result columns.");
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

        // Query
        Label queryLabel = new Label(container, SWT.NONE);
        queryLabel.setText("SQL Query:");
        queryLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        queryText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.H_SCROLL);
        GridData queryData = new GridData(SWT.FILL, SWT.FILL, true, true);
        queryData.heightHint = 150;
        queryText.setLayoutData(queryData);
        queryText.setMessage("SELECT id, name, email FROM members WHERE status = 'active'");

        // Description
        Label descLabel = new Label(container, SWT.NONE);
        descLabel.setText("Description:");

        descriptionText = new Text(container, SWT.BORDER);
        descriptionText.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        descriptionText.setMessage("Optional description of the query purpose");

        // Sample button
        new Label(container, SWT.NONE); // spacer

        Button sampleButton = new Button(container, SWT.PUSH);
        sampleButton.setText("Load Sample");
        sampleButton.addSelectionListener(new SelectionAdapter() {
            @Override
            public void widgetSelected(SelectionEvent e) {
                loadSampleData();
            }
        });

        return area;
    }

    private void loadSampleData() {
        queryText.setText(
                "SELECT m.id, m.name, m.email, m.phone, m.status, m.created_at\n" +
                        "FROM members m\n" +
                        "WHERE m.status = 'active'\n" +
                        "ORDER BY m.created_at DESC");
        descriptionText.setText("Active member list");
    }

    @Override
    protected void okPressed() {
        if (validateInput()) {
            buildQueryInput();
            super.okPressed();
        }
    }

    private boolean validateInput() {
        String query = queryText.getText().trim();
        if (query.isEmpty()) {
            setErrorMessage("SQL query is required.");
            return false;
        }

        if (!query.toUpperCase().startsWith("SELECT")) {
            setErrorMessage("Only SELECT queries are supported.");
            return false;
        }

        setErrorMessage(null);
        return true;
    }

    private void buildQueryInput() {
        queryInput = new QuerySampleInput(queryText.getText().trim());

        String desc = descriptionText.getText().trim();
        if (!desc.isEmpty()) {
            queryInput.setDescription(desc);
        }
    }

    public QuerySampleInput getQueryInput() {
        return queryInput;
    }

    @Override
    protected boolean isResizable() {
        return true;
    }
}
