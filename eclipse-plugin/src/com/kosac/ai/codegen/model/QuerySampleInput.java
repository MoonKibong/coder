package com.kosac.ai.codegen.model;

import java.util.HashMap;
import java.util.Map;

/**
 * SQL query sample input for code generation.
 */
public class QuerySampleInput {

    private String query;
    private String description;

    public QuerySampleInput() {
    }

    public QuerySampleInput(String query) {
        this.query = query;
    }

    public String getQuery() {
        return query;
    }

    public void setQuery(String query) {
        this.query = query;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("type", "query_sample");
        map.put("query", query);
        if (description != null) {
            map.put("description", description);
        }
        return map;
    }
}
