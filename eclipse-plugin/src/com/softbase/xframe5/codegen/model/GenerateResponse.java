package com.softbase.xframe5.codegen.model;

import java.util.List;

/**
 * Response from the code generation server.
 *
 * IMPORTANT: This response intentionally does NOT include:
 * - Model name
 * - Temperature
 * - Prompt used
 * - LLM configuration
 *
 * The server abstracts all LLM details from the plugin.
 */
public class GenerateResponse {

    private String status;
    private Artifacts artifacts;
    private List<String> warnings;
    private String error;
    private ResponseMeta meta;

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public Artifacts getArtifacts() {
        return artifacts;
    }

    public void setArtifacts(Artifacts artifacts) {
        this.artifacts = artifacts;
    }

    public List<String> getWarnings() {
        return warnings;
    }

    public void setWarnings(List<String> warnings) {
        this.warnings = warnings;
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
        return "success".equals(status) || "partial_success".equals(status);
    }

    public boolean hasWarnings() {
        return warnings != null && !warnings.isEmpty();
    }

    /**
     * Generated artifacts (XML and JavaScript).
     */
    public static class Artifacts {
        private String xml;
        private String javascript;

        public String getXml() {
            return xml;
        }

        public void setXml(String xml) {
            this.xml = xml;
        }

        public String getJavascript() {
            return javascript;
        }

        public void setJavascript(String javascript) {
            this.javascript = javascript;
        }
    }

    /**
     * Response metadata.
     * Note: This does NOT expose LLM model or configuration.
     */
    public static class ResponseMeta {
        private String generator;
        private String timestamp;
        private long generationTimeMs;

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

        public long getGenerationTimeMs() {
            return generationTimeMs;
        }

        public void setGenerationTimeMs(long generationTimeMs) {
            this.generationTimeMs = generationTimeMs;
        }
    }
}
