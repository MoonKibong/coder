package com.kosac.ai.codegen.dialogs;

import org.eclipse.jface.dialogs.TitleAreaDialog;
import org.eclipse.swt.SWT;
import org.eclipse.swt.graphics.Color;
import org.eclipse.swt.layout.GridData;
import org.eclipse.swt.layout.GridLayout;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.swt.widgets.Control;
import org.eclipse.swt.widgets.Label;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.swt.widgets.Text;

import com.kosac.ai.codegen.model.CodeReviewResponse;
import com.kosac.ai.codegen.model.CodeReviewResponse.ReviewIssue;
import com.kosac.ai.codegen.model.CodeReviewResponse.ReviewResult;

/**
 * Dialog for displaying code review results.
 */
public class CodeReviewResultDialog extends TitleAreaDialog {

    private CodeReviewResponse response;

    public CodeReviewResultDialog(Shell parentShell, CodeReviewResponse response) {
        super(parentShell);
        this.response = response;
        setShellStyle(getShellStyle() | SWT.RESIZE | SWT.MAX);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Code Review Results");

        if (response.isSuccess() && response.getReview() != null) {
            ReviewResult review = response.getReview();
            if (review.getScore() != null) {
                setMessage(String.format("Score: %d/100 (%s) - %d issues found",
                    review.getScore().getOverall(),
                    review.getScore().getRating(),
                    review.getIssueCount()));
            } else {
                setMessage(String.format("%d issues found", review.getIssueCount()));
            }
        } else {
            setMessage("Review completed with errors.");
        }
    }

    @Override
    protected Control createDialogArea(Composite parent) {
        Composite area = (Composite) super.createDialogArea(parent);

        Composite container = new Composite(area, SWT.NONE);
        container.setLayoutData(new GridData(SWT.FILL, SWT.FILL, true, true));
        GridLayout layout = new GridLayout(1, false);
        layout.marginHeight = 10;
        layout.marginWidth = 10;
        container.setLayout(layout);

        if (response.hasError()) {
            createErrorSection(container);
            return area;
        }

        ReviewResult review = response.getReview();
        if (review == null) {
            Label noResultLabel = new Label(container, SWT.NONE);
            noResultLabel.setText("No review results available.");
            return area;
        }

        // Summary section
        createSummarySection(container, review);

        // Score section
        if (review.getScore() != null) {
            createScoreSection(container, review);
        }

        // Issues section
        if (review.hasIssues()) {
            createIssuesSection(container, review);
        }

        // Improvements section
        if (review.getImprovements() != null && !review.getImprovements().isEmpty()) {
            createImprovementsSection(container, review);
        }

        return area;
    }

    private void createErrorSection(Composite parent) {
        Label errorLabel = new Label(parent, SWT.NONE);
        errorLabel.setText("Error:");
        errorLabel.setForeground(parent.getDisplay().getSystemColor(SWT.COLOR_RED));

        Text errorText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP | SWT.BORDER);
        GridData errorData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        errorData.heightHint = 60;
        errorText.setLayoutData(errorData);
        errorText.setText(response.getError());
        errorText.setForeground(parent.getDisplay().getSystemColor(SWT.COLOR_RED));
    }

    private void createSummarySection(Composite parent, ReviewResult review) {
        Label summaryLabel = new Label(parent, SWT.NONE);
        summaryLabel.setText("Summary:");
        summaryLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text summaryText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP | SWT.BORDER);
        GridData summaryData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        summaryData.heightHint = 60;
        summaryText.setLayoutData(summaryData);
        summaryText.setText(review.getSummary() != null ? review.getSummary() : "No summary available.");
    }

    private void createScoreSection(Composite parent, ReviewResult review) {
        Label scoreLabel = new Label(parent, SWT.NONE);
        scoreLabel.setText("Score Details:");
        scoreLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Composite scoreContainer = new Composite(parent, SWT.BORDER);
        scoreContainer.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        scoreContainer.setLayout(new GridLayout(4, false));

        // Overall score
        Label overallLabel = new Label(scoreContainer, SWT.NONE);
        overallLabel.setText("Overall: ");
        Label overallValue = new Label(scoreContainer, SWT.NONE);
        int overall = review.getScore().getOverall();
        overallValue.setText(overall + "/100 (" + review.getScore().getRating() + ")");
        overallValue.setForeground(getScoreColor(scoreContainer.getDisplay(), overall));

        // Category scores
        if (review.getScore().getCategories() != null) {
            CodeReviewResponse.CategoryScores cats = review.getScore().getCategories();

            if (cats.getSyntax() != null) {
                addScoreItem(scoreContainer, "Syntax", cats.getSyntax());
            }
            if (cats.getPatterns() != null) {
                addScoreItem(scoreContainer, "Patterns", cats.getPatterns());
            }
            if (cats.getNaming() != null) {
                addScoreItem(scoreContainer, "Naming", cats.getNaming());
            }
            if (cats.getPerformance() != null) {
                addScoreItem(scoreContainer, "Performance", cats.getPerformance());
            }
            if (cats.getSecurity() != null) {
                addScoreItem(scoreContainer, "Security", cats.getSecurity());
            }
        }
    }

    private void addScoreItem(Composite parent, String name, int value) {
        Label nameLabel = new Label(parent, SWT.NONE);
        nameLabel.setText(name + ": ");
        Label valueLabel = new Label(parent, SWT.NONE);
        valueLabel.setText(String.valueOf(value));
        valueLabel.setForeground(getScoreColor(parent.getDisplay(), value));
    }

    private Color getScoreColor(org.eclipse.swt.widgets.Display display, int score) {
        if (score >= 80) {
            return display.getSystemColor(SWT.COLOR_DARK_GREEN);
        } else if (score >= 60) {
            return display.getSystemColor(SWT.COLOR_DARK_YELLOW);
        } else {
            return display.getSystemColor(SWT.COLOR_RED);
        }
    }

    private void createIssuesSection(Composite parent, ReviewResult review) {
        Label issuesLabel = new Label(parent, SWT.NONE);
        issuesLabel.setText(String.format("Issues (%d errors, %d warnings):",
            review.getErrorCount(), review.getWarningCount()));
        issuesLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text issuesText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.V_SCROLL | SWT.H_SCROLL | SWT.BORDER);
        GridData issuesData = new GridData(SWT.FILL, SWT.FILL, true, true);
        issuesData.heightHint = 200;
        issuesText.setLayoutData(issuesData);

        StringBuilder sb = new StringBuilder();
        for (ReviewIssue issue : review.getIssues()) {
            sb.append(issue.toDisplayString()).append("\n\n");
        }
        issuesText.setText(sb.toString().trim());
    }

    private void createImprovementsSection(Composite parent, ReviewResult review) {
        Label improvementsLabel = new Label(parent, SWT.NONE);
        improvementsLabel.setText("Suggested Improvements:");
        improvementsLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text improvementsText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP | SWT.BORDER);
        GridData improvementsData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        improvementsData.heightHint = 80;
        improvementsText.setLayoutData(improvementsData);

        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < review.getImprovements().size(); i++) {
            sb.append(i + 1).append(". ").append(review.getImprovements().get(i)).append("\n");
        }
        improvementsText.setText(sb.toString().trim());
    }

    @Override
    protected void createButtonsForButtonBar(Composite parent) {
        createButton(parent, OK, "Close", true);
    }

    @Override
    protected boolean isResizable() {
        return true;
    }
}
