package com.kosac.ai.codegen.model;

import java.util.HashMap;
import java.util.Map;

/**
 * Request for Q&A.
 *
 * IMPORTANT: This request intentionally does NOT include:
 * - Model name
 * - Temperature
 * - LLM configuration
 *
 * The server decides all LLM details based on product.
 */
public class QARequest {

    private String product;
    private QAInput input;
    private QAOptions options;

    public QARequest() {
        this.options = new QAOptions();
    }

    public String getProduct() {
        return product;
    }

    public void setProduct(String product) {
        this.product = product;
    }

    public QAInput getInput() {
        return input;
    }

    public void setInput(QAInput input) {
        this.input = input;
    }

    public QAOptions getOptions() {
        return options;
    }

    public void setOptions(QAOptions options) {
        this.options = options;
    }

    /**
     * Create request for xFrame5 Q&A.
     */
    public static QARequest forXFrame5(String question) {
        QARequest request = new QARequest();
        request.setProduct("xframe5-ui");

        QAInput input = new QAInput();
        input.setQuestion(question);
        request.setInput(input);

        return request;
    }

    /**
     * Create request for Spring Q&A.
     */
    public static QARequest forSpring(String question) {
        QARequest request = new QARequest();
        request.setProduct("spring-backend");

        QAInput input = new QAInput();
        input.setQuestion(question);
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
        return map;
    }

    /**
     * Q&A input containing the question.
     */
    public static class QAInput {
        private String question;
        private String context;

        public String getQuestion() {
            return question;
        }

        public void setQuestion(String question) {
            this.question = question;
        }

        public String getContext() {
            return context;
        }

        public void setContext(String context) {
            this.context = context;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            map.put("question", question);
            if (context != null && !context.isEmpty()) {
                map.put("context", context);
            }
            return map;
        }
    }

    /**
     * Q&A options.
     */
    public static class QAOptions {
        private String language = "ko";
        private boolean includeExamples = true;
        private int maxReferences = 5;

        public String getLanguage() {
            return language;
        }

        public void setLanguage(String language) {
            this.language = language;
        }

        public boolean isIncludeExamples() {
            return includeExamples;
        }

        public void setIncludeExamples(boolean includeExamples) {
            this.includeExamples = includeExamples;
        }

        public int getMaxReferences() {
            return maxReferences;
        }

        public void setMaxReferences(int maxReferences) {
            this.maxReferences = maxReferences;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            map.put("language", language);
            map.put("include_examples", includeExamples);
            map.put("max_references", maxReferences);
            return map;
        }
    }
}
