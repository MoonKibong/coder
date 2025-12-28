package com.softbase.xframe5.codegen.model;

import java.util.HashMap;
import java.util.Map;

/**
 * Generation options.
 */
public class GenerateOptions {

    private String language = "ko";
    private boolean strictMode = false;
    private String companyId;

    public String getLanguage() {
        return language;
    }

    public void setLanguage(String language) {
        this.language = language;
    }

    public boolean isStrictMode() {
        return strictMode;
    }

    public void setStrictMode(boolean strictMode) {
        this.strictMode = strictMode;
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
        map.put("strict_mode", strictMode);
        if (companyId != null) {
            map.put("company_id", companyId);
        }
        return map;
    }
}
