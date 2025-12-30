package com.kosac.ai.codegen.model;

import java.util.List;

/**
 * Response from the Q&A server.
 *
 * IMPORTANT: This response intentionally does NOT include:
 * - Model name
 * - Temperature
 * - Prompt used
 * - LLM configuration
 *
 * The server abstracts all LLM details from the plugin.
 */
public class QAResponse {

    private String status;
    private QAAnswer answer;
    private List<KnowledgeReference> references;
    private String error;
    private ResponseMeta meta;

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public QAAnswer getAnswer() {
        return answer;
    }

    public void setAnswer(QAAnswer answer) {
        this.answer = answer;
    }

    public List<KnowledgeReference> getReferences() {
        return references;
    }

    public void setReferences(List<KnowledgeReference> references) {
        this.references = references;
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

    public boolean hasReferences() {
        return references != null && !references.isEmpty();
    }

    /**
     * The answer with text and code examples.
     */
    public static class QAAnswer {
        private String text;
        private List<CodeExample> codeExamples;
        private List<String> relatedTopics;

        public String getText() {
            return text;
        }

        public void setText(String text) {
            this.text = text;
        }

        public List<CodeExample> getCodeExamples() {
            return codeExamples;
        }

        public void setCodeExamples(List<CodeExample> codeExamples) {
            this.codeExamples = codeExamples;
        }

        public List<String> getRelatedTopics() {
            return relatedTopics;
        }

        public void setRelatedTopics(List<String> relatedTopics) {
            this.relatedTopics = relatedTopics;
        }

        public boolean hasCodeExamples() {
            return codeExamples != null && !codeExamples.isEmpty();
        }

        public boolean hasRelatedTopics() {
            return relatedTopics != null && !relatedTopics.isEmpty();
        }
    }

    /**
     * A code example in the answer.
     */
    public static class CodeExample {
        private String language;
        private String code;
        private String description;

        public String getLanguage() {
            return language;
        }

        public void setLanguage(String language) {
            this.language = language;
        }

        public String getCode() {
            return code;
        }

        public void setCode(String code) {
            this.code = code;
        }

        public String getDescription() {
            return description;
        }

        public void setDescription(String description) {
            this.description = description;
        }

        public boolean hasDescription() {
            return description != null && !description.isEmpty();
        }
    }

    /**
     * Reference to a knowledge base entry.
     */
    public static class KnowledgeReference {
        private int knowledgeId;
        private String name;
        private String category;
        private String section;
        private float relevance;

        public int getKnowledgeId() {
            return knowledgeId;
        }

        public void setKnowledgeId(int knowledgeId) {
            this.knowledgeId = knowledgeId;
        }

        public String getName() {
            return name;
        }

        public void setName(String name) {
            this.name = name;
        }

        public String getCategory() {
            return category;
        }

        public void setCategory(String category) {
            this.category = category;
        }

        public String getSection() {
            return section;
        }

        public void setSection(String section) {
            this.section = section;
        }

        public float getRelevance() {
            return relevance;
        }

        public void setRelevance(float relevance) {
            this.relevance = relevance;
        }

        /**
         * Get relevance as percentage string.
         */
        public String getRelevancePercent() {
            return String.format("%.0f%%", relevance * 100);
        }
    }

    /**
     * Response metadata.
     * Note: This does NOT expose LLM model or configuration.
     */
    public static class ResponseMeta {
        private String generator;
        private String timestamp;
        private long answerTimeMs;

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

        public long getAnswerTimeMs() {
            return answerTimeMs;
        }

        public void setAnswerTimeMs(long answerTimeMs) {
            this.answerTimeMs = answerTimeMs;
        }
    }
}
