package com.kosac.ai.codegen.client;

/**
 * Response from job status polling.
 */
public class JobStatusResponse {
    private String jobId;
    private String status;
    private Long queuePosition;
    private Long estimatedWaitSecs;
    private String artifacts;  // JSON string
    private String[] warnings;
    private String error;
    private Integer generationTimeMs;
    private String product;

    public String getJobId() {
        return jobId;
    }

    public void setJobId(String jobId) {
        this.jobId = jobId;
    }

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public Long getQueuePosition() {
        return queuePosition;
    }

    public void setQueuePosition(Long queuePosition) {
        this.queuePosition = queuePosition;
    }

    public Long getEstimatedWaitSecs() {
        return estimatedWaitSecs;
    }

    public void setEstimatedWaitSecs(Long estimatedWaitSecs) {
        this.estimatedWaitSecs = estimatedWaitSecs;
    }

    public String getArtifacts() {
        return artifacts;
    }

    public void setArtifacts(String artifacts) {
        this.artifacts = artifacts;
    }

    public String[] getWarnings() {
        return warnings;
    }

    public void setWarnings(String[] warnings) {
        this.warnings = warnings;
    }

    public String getError() {
        return error;
    }

    public void setError(String error) {
        this.error = error;
    }

    public Integer getGenerationTimeMs() {
        return generationTimeMs;
    }

    public void setGenerationTimeMs(Integer generationTimeMs) {
        this.generationTimeMs = generationTimeMs;
    }

    public String getProduct() {
        return product;
    }

    public void setProduct(String product) {
        this.product = product;
    }

    /**
     * Check if the job is still pending (queued or processing).
     */
    public boolean isPending() {
        return "queued".equals(status) || "processing".equals(status);
    }

    /**
     * Check if the job completed successfully.
     */
    public boolean isCompleted() {
        return "completed".equals(status);
    }

    /**
     * Check if the job failed.
     */
    public boolean isFailed() {
        return "failed".equals(status) || "cancelled".equals(status);
    }
}
