package com.kosac.ai.codegen.model;

import java.util.List;

/**
 * Response from the code review server.
 *
 * IMPORTANT: This response intentionally does NOT include:
 * - Model name
 * - Temperature
 * - Prompt used
 * - LLM configuration
 *
 * The server abstracts all LLM details from the plugin.
 */
public class CodeReviewResponse {

    private String status;
    private ReviewResult review;
    private String error;
    private ResponseMeta meta;

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public ReviewResult getReview() {
        return review;
    }

    public void setReview(ReviewResult review) {
        this.review = review;
    }

    public String getError() {
        return error;
    }

    public void setError(String error) {
        this.error = error;
    }

    public ResponseMeta getMeta() {
        return meta;
    }

    public void setMeta(ResponseMeta meta) {
        this.meta = meta;
    }

    public boolean isSuccess() {
        return "success".equals(status);
    }

    public boolean hasError() {
        return error != null && !error.isEmpty();
    }

    /**
     * Review result containing issues and suggestions.
     */
    public static class ReviewResult {
        private String summary;
        private List<ReviewIssue> issues;
        private ReviewScore score;
        private List<String> improvements;

        public String getSummary() {
            return summary;
        }

        public void setSummary(String summary) {
            this.summary = summary;
        }

        public List<ReviewIssue> getIssues() {
            return issues;
        }

        public void setIssues(List<ReviewIssue> issues) {
            this.issues = issues;
        }

        public ReviewScore getScore() {
            return score;
        }

        public void setScore(ReviewScore score) {
            this.score = score;
        }

        public List<String> getImprovements() {
            return improvements;
        }

        public void setImprovements(List<String> improvements) {
            this.improvements = improvements;
        }

        public boolean hasIssues() {
            return issues != null && !issues.isEmpty();
        }

        public int getIssueCount() {
            return issues != null ? issues.size() : 0;
        }

        public int getErrorCount() {
            if (issues == null) return 0;
            return (int) issues.stream()
                .filter(i -> "error".equals(i.getSeverity()))
                .count();
        }

        public int getWarningCount() {
            if (issues == null) return 0;
            return (int) issues.stream()
                .filter(i -> "warning".equals(i.getSeverity()))
                .count();
        }
    }

    /**
     * Individual review issue.
     */
    public static class ReviewIssue {
        private String severity;    // error, warning, info, suggestion
        private String category;    // syntax, pattern, naming, performance, security, best_practice
        private int line;
        private String message;
        private String suggestion;

        public String getSeverity() {
            return severity;
        }

        public void setSeverity(String severity) {
            this.severity = severity;
        }

        public String getCategory() {
            return category;
        }

        public void setCategory(String category) {
            this.category = category;
        }

        public int getLine() {
            return line;
        }

        public void setLine(int line) {
            this.line = line;
        }

        public String getMessage() {
            return message;
        }

        public void setMessage(String message) {
            this.message = message;
        }

        public String getSuggestion() {
            return suggestion;
        }

        public void setSuggestion(String suggestion) {
            this.suggestion = suggestion;
        }

        public boolean hasSuggestion() {
            return suggestion != null && !suggestion.isEmpty();
        }

        /**
         * Get formatted issue string for display.
         */
        public String toDisplayString() {
            StringBuilder sb = new StringBuilder();
            sb.append("[").append(severity.toUpperCase()).append("]");
            if (line > 0) {
                sb.append(" Line ").append(line).append(":");
            }
            sb.append(" ").append(message);
            if (hasSuggestion()) {
                sb.append("\n   Suggestion: ").append(suggestion);
            }
            return sb.toString();
        }
    }

    /**
     * Review score.
     */
    public static class ReviewScore {
        private int overall;
        private CategoryScores categories;

        public int getOverall() {
            return overall;
        }

        public void setOverall(int overall) {
            this.overall = overall;
        }

        public CategoryScores getCategories() {
            return categories;
        }

        public void setCategories(CategoryScores categories) {
            this.categories = categories;
        }

        /**
         * Get score rating as text.
         */
        public String getRating() {
            if (overall >= 90) return "Excellent";
            if (overall >= 80) return "Good";
            if (overall >= 70) return "Fair";
            if (overall >= 60) return "Needs Improvement";
            return "Poor";
        }
    }

    /**
     * Category-specific scores.
     */
    public static class CategoryScores {
        private Integer syntax;
        private Integer patterns;
        private Integer naming;
        private Integer performance;
        private Integer security;

        public Integer getSyntax() {
            return syntax;
        }

        public void setSyntax(Integer syntax) {
            this.syntax = syntax;
        }

        public Integer getPatterns() {
            return patterns;
        }

        public void setPatterns(Integer patterns) {
            this.patterns = patterns;
        }

        public Integer getNaming() {
            return naming;
        }

        public void setNaming(Integer naming) {
            this.naming = naming;
        }

        public Integer getPerformance() {
            return performance;
        }

        public void setPerformance(Integer performance) {
            this.performance = performance;
        }

        public Integer getSecurity() {
            return security;
        }

        public void setSecurity(Integer security) {
            this.security = security;
        }
    }

    /**
     * Response metadata.
     * Note: This does NOT expose LLM model or configuration.
     */
    public static class ResponseMeta {
        private String generator;
        private String timestamp;
        private long reviewTimeMs;

        public String getGenerator() {
            return generator;
        }

        public void setGenerator(String generator) {
            this.generator = generator;
        }

        public String getTimestamp() {
            return timestamp;
        }

        public void setTimestamp(String timestamp) {
            this.timestamp = timestamp;
        }

        public long getReviewTimeMs() {
            return reviewTimeMs;
        }

        public void setReviewTimeMs(long reviewTimeMs) {
            this.reviewTimeMs = reviewTimeMs;
        }
    }
}
