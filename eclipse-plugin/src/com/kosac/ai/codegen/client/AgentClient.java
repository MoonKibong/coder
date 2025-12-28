package com.kosac.ai.codegen.client;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import org.eclipse.core.runtime.IProgressMonitor;

import com.kosac.ai.codegen.model.GenerateRequest;
import com.kosac.ai.codegen.model.GenerateResponse;
import com.kosac.ai.codegen.model.SpringGenerateResponse;

/**
 * HTTP client for communicating with the Enterprise Code Generator agent server.
 *
 * IMPORTANT: This client is intentionally "dumb" - it does NOT know:
 * - LLM model names (codellama, mistral, etc.)
 * - Temperature, token limits, or other LLM parameters
 * - Prompt templates or system prompts
 * - Whether the server uses Ollama, llama.cpp, or any other LLM backend
 *
 * It only sends:
 * - product (e.g., "xframe5-ui")
 * - input (schema, query, or natural language)
 * - options (language, strict mode)
 * - context (project info)
 */
public class AgentClient {

    private static final int DEFAULT_TIMEOUT = 120000; // 2 minutes for LLM generation

    private final String endpoint;
    private final int timeout;

    public AgentClient(String endpoint) {
        this(endpoint, DEFAULT_TIMEOUT);
    }

    public AgentClient(String endpoint, int timeout) {
        this.endpoint = endpoint.endsWith("/") ? endpoint.substring(0, endpoint.length() - 1) : endpoint;
        this.timeout = timeout;
    }

    /**
     * Generate code from the given request.
     *
     * @param request The generation request (contains NO LLM configuration)
     * @return The generation response with XML and JavaScript artifacts
     * @throws AgentClientException if the request fails
     */
    public GenerateResponse generate(GenerateRequest request) throws AgentClientException {
        try {
            String json = mapToJson(request.toMap());
            String responseBody = post("/agent/generate", json);
            return parseResponse(responseBody);
        } catch (IOException e) {
            throw new AgentClientException("Failed to communicate with agent server: " + e.getMessage(), e);
        }
    }

    /**
     * Generate Spring backend code from the given request.
     *
     * @param request The generation request with product="spring-backend"
     * @return The generation response with Spring artifacts
     * @throws AgentClientException if the request fails
     */
    public SpringGenerateResponse generateSpring(GenerateRequest request) throws AgentClientException {
        try {
            // Ensure product is set to spring-backend
            request.setProduct("spring-backend");
            String json = mapToJson(request.toMap());
            String responseBody = post("/agent/generate", json);
            return parseSpringResponse(responseBody);
        } catch (IOException e) {
            throw new AgentClientException("Failed to communicate with agent server: " + e.getMessage(), e);
        }
    }

    /**
     * Check if the agent server is available and healthy.
     *
     * @return true if the server is healthy
     * @throws AgentClientException if the health check fails
     */
    public boolean healthCheck() throws AgentClientException {
        try {
            String responseBody = get("/agent/health");
            // Parse response to check status
            if (responseBody.contains("\"status\"")) {
                return responseBody.contains("\"healthy\"") || responseBody.contains("\"degraded\"");
            }
            return false;
        } catch (IOException e) {
            throw new AgentClientException("Agent server health check failed: " + e.getMessage(), e);
        }
    }

    /**
     * Get available products from the server.
     *
     * @return JSON string containing available products
     * @throws AgentClientException if the request fails
     */
    public String getProducts() throws AgentClientException {
        try {
            return get("/agent/products");
        } catch (IOException e) {
            throw new AgentClientException("Failed to get products: " + e.getMessage(), e);
        }
    }

    /**
     * Submit a generation request asynchronously.
     *
     * @param request The generation request
     * @return AsyncJobResponse with job ID for polling
     * @throws AgentClientException if the request fails
     */
    public AsyncJobResponse generateAsync(GenerateRequest request) throws AgentClientException {
        try {
            String json = mapToJson(request.toMap());
            String responseBody = post("/agent/generate?mode=async", json);
            return parseAsyncJobResponse(responseBody);
        } catch (IOException e) {
            throw new AgentClientException("Failed to submit async job: " + e.getMessage(), e);
        }
    }

    /**
     * Get job status by ID.
     *
     * @param jobId The job ID to check
     * @return JobStatusResponse with current status
     * @throws AgentClientException if the request fails
     */
    public JobStatusResponse getJobStatus(String jobId) throws AgentClientException {
        try {
            String responseBody = get("/agent/jobs/" + jobId);
            return parseJobStatusResponse(responseBody);
        } catch (IOException e) {
            throw new AgentClientException("Failed to get job status: " + e.getMessage(), e);
        }
    }

    /**
     * Poll for job completion with progress reporting.
     *
     * @param jobId The job ID to poll
     * @param monitor Progress monitor for UI updates (can be null)
     * @param pollIntervalMs Polling interval in milliseconds
     * @param maxWaitMs Maximum wait time in milliseconds
     * @return JobStatusResponse when completed or failed
     * @throws AgentClientException if polling fails or times out
     */
    public JobStatusResponse pollUntilComplete(
            String jobId,
            IProgressMonitor monitor,
            long pollIntervalMs,
            long maxWaitMs) throws AgentClientException {

        long startTime = System.currentTimeMillis();

        while (true) {
            // Check for cancellation
            if (monitor != null && monitor.isCanceled()) {
                throw new AgentClientException("Operation cancelled by user");
            }

            // Check timeout
            if (System.currentTimeMillis() - startTime > maxWaitMs) {
                throw new AgentClientException("Job timed out after " + (maxWaitMs / 1000) + " seconds");
            }

            // Get status
            JobStatusResponse status = getJobStatus(jobId);

            // Update progress
            if (monitor != null) {
                if (status.getQueuePosition() != null) {
                    monitor.subTask("Position in queue: " + status.getQueuePosition());
                } else if ("processing".equals(status.getStatus())) {
                    monitor.subTask("Generating code...");
                }
            }

            // Check if done
            if (!status.isPending()) {
                return status;
            }

            // Wait before next poll
            try {
                Thread.sleep(pollIntervalMs);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new AgentClientException("Polling interrupted");
            }
        }
    }

    /**
     * Generate code asynchronously with automatic polling.
     * This is the recommended method for UI handlers.
     *
     * @param request The generation request
     * @param monitor Progress monitor for UI updates
     * @return GenerateResponse when completed
     * @throws AgentClientException if generation fails
     */
    public GenerateResponse generateWithPolling(GenerateRequest request, IProgressMonitor monitor)
            throws AgentClientException {
        // Submit async job
        if (monitor != null) {
            monitor.subTask("Submitting job to queue...");
        }
        AsyncJobResponse asyncResponse = generateAsync(request);

        if (monitor != null) {
            monitor.subTask("Job queued: " + asyncResponse.getJobId());
        }

        // Poll until complete
        JobStatusResponse status = pollUntilComplete(
                asyncResponse.getJobId(),
                monitor,
                2000,  // Poll every 2 seconds
                timeout  // Use configured timeout
        );

        // Convert to GenerateResponse
        if (status.isCompleted()) {
            GenerateResponse response = new GenerateResponse();
            response.setStatus("success");
            if (status.getArtifacts() != null) {
                GenerateResponse.Artifacts artifacts = parseArtifacts(status.getArtifacts());
                response.setArtifacts(artifacts);
            }
            return response;
        } else {
            GenerateResponse response = new GenerateResponse();
            response.setStatus("error");
            response.setError(status.getError() != null ? status.getError() : "Job failed");
            return response;
        }
    }

    /**
     * Generate Spring code asynchronously with automatic polling.
     *
     * @param request The generation request
     * @param monitor Progress monitor for UI updates
     * @return SpringGenerateResponse when completed
     * @throws AgentClientException if generation fails
     */
    public SpringGenerateResponse generateSpringWithPolling(GenerateRequest request, IProgressMonitor monitor)
            throws AgentClientException {
        request.setProduct("spring-backend");

        // Submit async job
        if (monitor != null) {
            monitor.subTask("Submitting Spring generation job...");
        }
        AsyncJobResponse asyncResponse = generateAsync(request);

        if (monitor != null) {
            monitor.subTask("Job queued: " + asyncResponse.getJobId());
        }

        // Poll until complete
        JobStatusResponse status = pollUntilComplete(
                asyncResponse.getJobId(),
                monitor,
                2000,
                timeout
        );

        // Convert to SpringGenerateResponse
        if (status.isCompleted()) {
            SpringGenerateResponse response = new SpringGenerateResponse();
            response.setStatus("success");
            if (status.getArtifacts() != null) {
                SpringGenerateResponse.Artifacts artifacts = parseSpringArtifacts(status.getArtifacts());
                response.setArtifacts(artifacts);
            }
            return response;
        } else {
            SpringGenerateResponse response = new SpringGenerateResponse();
            response.setStatus("error");
            response.setError(status.getError() != null ? status.getError() : "Job failed");
            return response;
        }
    }

    private GenerateResponse.Artifacts parseArtifacts(String json) {
        GenerateResponse.Artifacts artifacts = new GenerateResponse.Artifacts();
        artifacts.setXml(extractJsonString(json, "xml"));
        artifacts.setJavascript(extractJsonString(json, "javascript"));
        return artifacts;
    }

    private SpringGenerateResponse.Artifacts parseSpringArtifacts(String json) {
        SpringGenerateResponse.Artifacts artifacts = new SpringGenerateResponse.Artifacts();
        artifacts.setController(extractJsonString(json, "controller"));
        artifacts.setServiceInterface(extractJsonString(json, "service_interface"));
        artifacts.setServiceImpl(extractJsonString(json, "service_impl"));
        artifacts.setDto(extractJsonString(json, "dto"));
        artifacts.setSearchDto(extractJsonString(json, "search_dto"));
        artifacts.setMapperInterface(extractJsonString(json, "mapper_interface"));
        artifacts.setMapperXml(extractJsonString(json, "mapper_xml"));
        return artifacts;
    }

    private AsyncJobResponse parseAsyncJobResponse(String json) {
        AsyncJobResponse response = new AsyncJobResponse();
        response.setJobId(extractJsonString(json, "job_id"));
        response.setStatus(extractJsonString(json, "status"));
        response.setStatusUrl(extractJsonString(json, "status_url"));
        response.setMessage(extractJsonString(json, "message"));
        return response;
    }

    private JobStatusResponse parseJobStatusResponse(String json) {
        JobStatusResponse response = new JobStatusResponse();
        response.setJobId(extractJsonString(json, "job_id"));
        response.setStatus(extractJsonString(json, "status"));
        response.setError(extractJsonString(json, "error"));
        response.setProduct(extractJsonString(json, "product"));
        response.setArtifacts(extractJsonObject(json, "artifacts"));

        String queuePos = extractJsonString(json, "queue_position");
        if (queuePos != null) {
            try {
                response.setQueuePosition(Long.parseLong(queuePos));
            } catch (NumberFormatException e) {
                // Ignore
            }
        }

        String genTime = extractJsonString(json, "generation_time_ms");
        if (genTime != null) {
            try {
                response.setGenerationTimeMs(Integer.parseInt(genTime));
            } catch (NumberFormatException e) {
                // Ignore
            }
        }

        return response;
    }

    private String extractJsonObject(String json, String key) {
        String searchKey = "\"" + key + "\"";
        int keyStart = json.indexOf(searchKey);
        if (keyStart < 0) {
            return null;
        }

        int colonPos = json.indexOf(":", keyStart + searchKey.length());
        if (colonPos < 0) {
            return null;
        }

        // Skip whitespace
        int valueStart = colonPos + 1;
        while (valueStart < json.length() && Character.isWhitespace(json.charAt(valueStart))) {
            valueStart++;
        }

        if (valueStart >= json.length() || json.charAt(valueStart) != '{') {
            return null;
        }

        int objEnd = findMatchingBrace(json, valueStart);
        if (objEnd > valueStart) {
            return json.substring(valueStart, objEnd + 1);
        }
        return null;
    }

    private String post(String path, String body) throws IOException {
        URL url = new URL(endpoint + path);
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();

        try {
            conn.setRequestMethod("POST");
            conn.setRequestProperty("Content-Type", "application/json; charset=UTF-8");
            conn.setRequestProperty("Accept", "application/json");
            conn.setConnectTimeout(10000); // 10 seconds to connect
            conn.setReadTimeout(timeout);
            conn.setDoOutput(true);

            try (OutputStream os = conn.getOutputStream()) {
                byte[] input = body.getBytes(StandardCharsets.UTF_8);
                os.write(input, 0, input.length);
            }

            int responseCode = conn.getResponseCode();
            if (responseCode >= 200 && responseCode < 300) {
                return readResponse(conn);
            } else {
                String errorBody = readErrorResponse(conn);
                throw new IOException("Server returned error " + responseCode + ": " + errorBody);
            }
        } finally {
            conn.disconnect();
        }
    }

    private String get(String path) throws IOException {
        URL url = new URL(endpoint + path);
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();

        try {
            conn.setRequestMethod("GET");
            conn.setRequestProperty("Accept", "application/json");
            conn.setConnectTimeout(10000);
            conn.setReadTimeout(30000);

            int responseCode = conn.getResponseCode();
            if (responseCode >= 200 && responseCode < 300) {
                return readResponse(conn);
            } else {
                String errorBody = readErrorResponse(conn);
                throw new IOException("Server returned error " + responseCode + ": " + errorBody);
            }
        } finally {
            conn.disconnect();
        }
    }

    private String readResponse(HttpURLConnection conn) throws IOException {
        try (BufferedReader br = new BufferedReader(
                new InputStreamReader(conn.getInputStream(), StandardCharsets.UTF_8))) {
            return br.lines().collect(Collectors.joining("\n"));
        }
    }

    private String readErrorResponse(HttpURLConnection conn) {
        try (BufferedReader br = new BufferedReader(
                new InputStreamReader(conn.getErrorStream(), StandardCharsets.UTF_8))) {
            return br.lines().collect(Collectors.joining("\n"));
        } catch (Exception e) {
            return "Unable to read error response";
        }
    }

    /**
     * Simple JSON serialization for Map objects.
     * Note: For production, consider using a proper JSON library like Gson or Jackson.
     */
    @SuppressWarnings("unchecked")
    private String mapToJson(Map<String, Object> map) {
        StringBuilder sb = new StringBuilder();
        sb.append("{");

        boolean first = true;
        for (Map.Entry<String, Object> entry : map.entrySet()) {
            if (!first) {
                sb.append(",");
            }
            first = false;

            sb.append("\"").append(escapeJson(entry.getKey())).append("\":");
            sb.append(valueToJson(entry.getValue()));
        }

        sb.append("}");
        return sb.toString();
    }

    @SuppressWarnings("unchecked")
    private String valueToJson(Object value) {
        if (value == null) {
            return "null";
        } else if (value instanceof String) {
            return "\"" + escapeJson((String) value) + "\"";
        } else if (value instanceof Number) {
            return value.toString();
        } else if (value instanceof Boolean) {
            return value.toString();
        } else if (value instanceof List) {
            StringBuilder sb = new StringBuilder();
            sb.append("[");
            boolean first = true;
            for (Object item : (List<?>) value) {
                if (!first) {
                    sb.append(",");
                }
                first = false;
                sb.append(valueToJson(item));
            }
            sb.append("]");
            return sb.toString();
        } else if (value instanceof Map) {
            return mapToJson((Map<String, Object>) value);
        } else {
            return "\"" + escapeJson(value.toString()) + "\"";
        }
    }

    private String escapeJson(String text) {
        if (text == null) {
            return "";
        }
        return text
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
    }

    /**
     * Simple JSON parsing for response.
     * Note: For production, consider using a proper JSON library.
     */
    private GenerateResponse parseResponse(String json) {
        GenerateResponse response = new GenerateResponse();

        // Extract status
        response.setStatus(extractJsonString(json, "status"));

        // Extract error
        response.setError(extractJsonString(json, "error"));

        // Extract artifacts
        int artifactsStart = json.indexOf("\"artifacts\"");
        if (artifactsStart >= 0) {
            GenerateResponse.Artifacts artifacts = new GenerateResponse.Artifacts();
            // Find the artifacts object
            int objStart = json.indexOf("{", artifactsStart);
            if (objStart >= 0) {
                int objEnd = findMatchingBrace(json, objStart);
                if (objEnd > objStart) {
                    String artifactsJson = json.substring(objStart, objEnd + 1);
                    artifacts.setXml(extractJsonString(artifactsJson, "xml"));
                    artifacts.setJavascript(extractJsonString(artifactsJson, "javascript"));
                }
            }
            response.setArtifacts(artifacts);
        }

        // Extract meta
        int metaStart = json.indexOf("\"meta\"");
        if (metaStart >= 0) {
            GenerateResponse.ResponseMeta meta = new GenerateResponse.ResponseMeta();
            int objStart = json.indexOf("{", metaStart);
            if (objStart >= 0) {
                int objEnd = findMatchingBrace(json, objStart);
                if (objEnd > objStart) {
                    String metaJson = json.substring(objStart, objEnd + 1);
                    meta.setGenerator(extractJsonString(metaJson, "generator"));
                    meta.setTimestamp(extractJsonString(metaJson, "timestamp"));
                    String timeMs = extractJsonString(metaJson, "generation_time_ms");
                    if (timeMs != null) {
                        try {
                            meta.setGenerationTimeMs(Long.parseLong(timeMs));
                        } catch (NumberFormatException e) {
                            // Ignore
                        }
                    }
                }
            }
            response.setMeta(meta);
        }

        return response;
    }

    private String extractJsonString(String json, String key) {
        String searchKey = "\"" + key + "\"";
        int keyStart = json.indexOf(searchKey);
        if (keyStart < 0) {
            return null;
        }

        int colonPos = json.indexOf(":", keyStart + searchKey.length());
        if (colonPos < 0) {
            return null;
        }

        // Skip whitespace
        int valueStart = colonPos + 1;
        while (valueStart < json.length() && Character.isWhitespace(json.charAt(valueStart))) {
            valueStart++;
        }

        if (valueStart >= json.length()) {
            return null;
        }

        char firstChar = json.charAt(valueStart);

        // Handle null
        if (json.substring(valueStart).startsWith("null")) {
            return null;
        }

        // Handle string
        if (firstChar == '"') {
            int valueEnd = findEndOfString(json, valueStart);
            if (valueEnd > valueStart) {
                return unescapeJson(json.substring(valueStart + 1, valueEnd));
            }
        }

        // Handle number
        if (Character.isDigit(firstChar) || firstChar == '-') {
            int valueEnd = valueStart;
            while (valueEnd < json.length() &&
                    (Character.isDigit(json.charAt(valueEnd)) || json.charAt(valueEnd) == '.')) {
                valueEnd++;
            }
            return json.substring(valueStart, valueEnd);
        }

        return null;
    }

    private int findEndOfString(String json, int start) {
        boolean escaped = false;
        for (int i = start + 1; i < json.length(); i++) {
            char c = json.charAt(i);
            if (escaped) {
                escaped = false;
            } else if (c == '\\') {
                escaped = true;
            } else if (c == '"') {
                return i;
            }
        }
        return -1;
    }

    private int findMatchingBrace(String json, int start) {
        int depth = 0;
        boolean inString = false;
        boolean escaped = false;

        for (int i = start; i < json.length(); i++) {
            char c = json.charAt(i);

            if (escaped) {
                escaped = false;
                continue;
            }

            if (c == '\\' && inString) {
                escaped = true;
                continue;
            }

            if (c == '"') {
                inString = !inString;
                continue;
            }

            if (!inString) {
                if (c == '{') {
                    depth++;
                } else if (c == '}') {
                    depth--;
                    if (depth == 0) {
                        return i;
                    }
                }
            }
        }
        return -1;
    }

    private String unescapeJson(String text) {
        if (text == null) {
            return null;
        }
        return text
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");
    }

    /**
     * Parse Spring generation response.
     */
    private SpringGenerateResponse parseSpringResponse(String json) {
        SpringGenerateResponse response = new SpringGenerateResponse();

        // Extract status
        response.setStatus(extractJsonString(json, "status"));

        // Extract error
        response.setError(extractJsonString(json, "error"));

        // Extract artifacts
        int artifactsStart = json.indexOf("\"artifacts\"");
        if (artifactsStart >= 0) {
            SpringGenerateResponse.Artifacts artifacts = new SpringGenerateResponse.Artifacts();
            int objStart = json.indexOf("{", artifactsStart);
            if (objStart >= 0) {
                int objEnd = findMatchingBrace(json, objStart);
                if (objEnd > objStart) {
                    String artifactsJson = json.substring(objStart, objEnd + 1);
                    artifacts.setController(extractJsonString(artifactsJson, "controller"));
                    artifacts.setServiceInterface(extractJsonString(artifactsJson, "service_interface"));
                    artifacts.setServiceImpl(extractJsonString(artifactsJson, "service_impl"));
                    artifacts.setDto(extractJsonString(artifactsJson, "dto"));
                    artifacts.setSearchDto(extractJsonString(artifactsJson, "search_dto"));
                    artifacts.setMapperInterface(extractJsonString(artifactsJson, "mapper_interface"));
                    artifacts.setMapperXml(extractJsonString(artifactsJson, "mapper_xml"));
                }
            }
            response.setArtifacts(artifacts);
        }

        // Extract meta
        int metaStart = json.indexOf("\"meta\"");
        if (metaStart >= 0) {
            SpringGenerateResponse.Meta meta = new SpringGenerateResponse.Meta();
            int objStart = json.indexOf("{", metaStart);
            if (objStart >= 0) {
                int objEnd = findMatchingBrace(json, objStart);
                if (objEnd > objStart) {
                    String metaJson = json.substring(objStart, objEnd + 1);
                    meta.setGenerator(extractJsonString(metaJson, "generator"));
                    meta.setTimestamp(extractJsonString(metaJson, "timestamp"));
                    String timeMs = extractJsonString(metaJson, "generation_time_ms");
                    if (timeMs != null) {
                        try {
                            meta.setGenerationTimeMs(Long.parseLong(timeMs));
                        } catch (NumberFormatException e) {
                            // Ignore
                        }
                    }
                }
            }
            response.setMeta(meta);
        }

        return response;
    }
}
