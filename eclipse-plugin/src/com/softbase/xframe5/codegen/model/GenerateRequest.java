package com.softbase.xframe5.codegen.model;

import java.util.HashMap;
import java.util.Map;

/**
 * Request object for code generation.
 *
 * IMPORTANT: This class intentionally does NOT include:
 * - model (LLM model name)
 * - temperature
 * - prompt
 * - systemPrompt
 *
 * The plugin is "dumb" - it only sends input data, not LLM configuration.
 */
public class GenerateRequest {

    private String product = "xframe5-ui";
    private Object input;
    private GenerateOptions options;
    private RequestContext context;

    public GenerateRequest() {
        this.options = new GenerateOptions();
        this.context = new RequestContext();
    }

    public String getProduct() {
        return product;
    }

    public void setProduct(String product) {
        this.product = product;
    }

    public Object getInput() {
        return input;
    }

    public void setInput(Object input) {
        this.input = input;
    }

    public GenerateOptions getOptions() {
        return options;
    }

    public void setOptions(GenerateOptions options) {
        this.options = options;
    }

    public RequestContext getContext() {
        return context;
    }

    public void setContext(RequestContext context) {
        this.context = context;
    }

    /**
     * Create request for DB schema input
     */
    public static GenerateRequest fromSchema(SchemaInput schema) {
        GenerateRequest request = new GenerateRequest();
        request.setInput(schema.toMap());
        return request;
    }

    /**
     * Create request for query sample input
     */
    public static GenerateRequest fromQuery(QuerySampleInput query) {
        GenerateRequest request = new GenerateRequest();
        request.setInput(query.toMap());
        return request;
    }

    /**
     * Create request for natural language input
     */
    public static GenerateRequest fromNaturalLanguage(NaturalLanguageInput nl) {
        GenerateRequest request = new GenerateRequest();
        request.setInput(nl.toMap());
        return request;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("product", product);
        map.put("input", input);
        map.put("options", options.toMap());
        map.put("context", context.toMap());
        return map;
    }
}
