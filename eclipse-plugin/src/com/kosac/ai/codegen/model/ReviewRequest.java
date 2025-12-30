package com.kosac.ai.codegen.model;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Request for code review.
 *
 * IMPORTANT: This request intentionally does NOT include:
 * - Model name
 * - Temperature
 * - LLM configuration
 *
 * The server decides all LLM details based on product.
 */
public class ReviewRequest {

    private String product;
    private ReviewInput input;
    private ReviewOptions options;
    private ReviewContext context;

    public ReviewRequest() {
        this.options = new ReviewOptions();
        this.context = new ReviewContext();
    }

    public String getProduct() {
        return product;
    }

    public void setProduct(String product) {
        this.product = product;
    }

    public ReviewInput getInput() {
        return input;
    }

    public void setInput(ReviewInput input) {
        this.input = input;
    }

    public ReviewOptions getOptions() {
        return options;
    }

    public void setOptions(ReviewOptions options) {
        this.options = options;
    }

    public ReviewContext getContext() {
        return context;
    }

    public void setContext(ReviewContext context) {
        this.context = context;
    }

    /**
     * Create request for xFrame5 code review.
     */
    public static ReviewRequest forXFrame5(String code, String fileType) {
        ReviewRequest request = new ReviewRequest();
        request.setProduct("xframe5-ui");

        ReviewInput input = new ReviewInput();
        input.setCode(code);
        input.setFileType(fileType);
        request.setInput(input);

        return request;
    }

    /**
     * Create request for Spring code review.
     */
    public static ReviewRequest forSpring(String code, String fileType) {
        ReviewRequest request = new ReviewRequest();
        request.setProduct("spring-backend");

        ReviewInput input = new ReviewInput();
        input.setCode(code);
        input.setFileType(fileType);
        request.setInput(input);

        return request;
    }

    /**
     * Convert to map for JSON serialization.
     */
    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("product", product);
        map.put("input", input != null ? input.toMap() : new HashMap<>());
        map.put("options", options != null ? options.toMap() : new HashMap<>());
        map.put("context", context != null ? context.toMap() : new HashMap<>());
        return map;
    }

    /**
     * Review input containing code to review.
     */
    public static class ReviewInput {
        private String code;
        private String fileType;    // xml, javascript, java
        private String inputContext;    // optional additional context

        public String getCode() {
            return code;
        }

        public void setCode(String code) {
            this.code = code;
        }

        public String getFileType() {
            return fileType;
        }

        public void setFileType(String fileType) {
            this.fileType = fileType;
        }

        public String getContext() {
            return inputContext;
        }

        public void setContext(String context) {
            this.inputContext = context;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            map.put("code", code);
            if (fileType != null) {
                map.put("file_type", fileType);
            }
            if (inputContext != null) {
                map.put("context", inputContext);
            }
            return map;
        }
    }

    /**
     * Review options.
     */
    public static class ReviewOptions {
        private String language = "ko";
        private List<String> reviewFocus = Arrays.asList("syntax", "patterns", "naming", "performance");
        private String companyId;

        public String getLanguage() {
            return language;
        }

        public void setLanguage(String language) {
            this.language = language;
        }

        public List<String> getReviewFocus() {
            return reviewFocus;
        }

        public void setReviewFocus(List<String> reviewFocus) {
            this.reviewFocus = reviewFocus;
        }

        public String getCompanyId() {
            return companyId;
        }

        public void setCompanyId(String companyId) {
            this.companyId = companyId;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            map.put("language", language);
            map.put("review_focus", reviewFocus);
            if (companyId != null) {
                map.put("company_id", companyId);
            }
            return map;
        }
    }

    /**
     * Review context.
     */
    public static class ReviewContext {
        private String project;
        private String fileName;

        public String getProject() {
            return project;
        }

        public void setProject(String project) {
            this.project = project;
        }

        public String getFileName() {
            return fileName;
        }

        public void setFileName(String fileName) {
            this.fileName = fileName;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            if (project != null) {
                map.put("project", project);
            }
            if (fileName != null) {
                map.put("file_name", fileName);
            }
            return map;
        }
    }
}
