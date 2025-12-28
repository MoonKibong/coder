package com.softbase.xframe5.codegen.model;

import java.util.List;

/**
 * Response from agent server for Spring backend code generation.
 */
public class SpringGenerateResponse {

    private String status;
    private Artifacts artifacts;
    private List<String> warnings;
    private String error;
    private Meta meta;

    public boolean isSuccess() {
        return "success".equals(status) || "partial_success".equals(status);
    }

    public boolean hasWarnings() {
        return warnings != null && !warnings.isEmpty();
    }

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

    public Meta getMeta() {
        return meta;
    }

    public void setMeta(Meta meta) {
        this.meta = meta;
    }

    /**
     * Generated Spring artifacts.
     */
    public static class Artifacts {
        private String controller;
        private String serviceInterface;
        private String serviceImpl;
        private String dto;
        private String searchDto;
        private String mapperInterface;
        private String mapperXml;

        public String getController() {
            return controller;
        }

        public void setController(String controller) {
            this.controller = controller;
        }

        public String getServiceInterface() {
            return serviceInterface;
        }

        public void setServiceInterface(String serviceInterface) {
            this.serviceInterface = serviceInterface;
        }

        public String getServiceImpl() {
            return serviceImpl;
        }

        public void setServiceImpl(String serviceImpl) {
            this.serviceImpl = serviceImpl;
        }

        public String getDto() {
            return dto;
        }

        public void setDto(String dto) {
            this.dto = dto;
        }

        public String getSearchDto() {
            return searchDto;
        }

        public void setSearchDto(String searchDto) {
            this.searchDto = searchDto;
        }

        public String getMapperInterface() {
            return mapperInterface;
        }

        public void setMapperInterface(String mapperInterface) {
            this.mapperInterface = mapperInterface;
        }

        public String getMapperXml() {
            return mapperXml;
        }

        public void setMapperXml(String mapperXml) {
            this.mapperXml = mapperXml;
        }
    }

    /**
     * Response metadata.
     */
    public static class Meta {
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
