package com.kosac.ai.codegen.model;

import java.util.HashMap;
import java.util.Map;

/**
 * Natural language description input for code generation.
 */
public class NaturalLanguageInput {

    private String description;
    private String screenType;
    private String context;

    public NaturalLanguageInput() {
    }

    public NaturalLanguageInput(String description) {
        this.description = description;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public String getScreenType() {
        return screenType;
    }

    public void setScreenType(String screenType) {
        this.screenType = screenType;
    }

    public String getContext() {
        return context;
    }

    public void setContext(String context) {
        this.context = context;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("type", "natural_language");
        map.put("description", description);
        if (screenType != null) {
            map.put("screen_type", screenType);
        }
        if (context != null) {
            map.put("context", context);
        }
        return map;
    }
}
