package com.kosac.ai.codegen.model;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Request context containing project information.
 */
public class RequestContext {

    private String project;
    private String target = "frontend";
    private List<String> output = Arrays.asList("xml", "javascript");

    public String getProject() {
        return project;
    }

    public void setProject(String project) {
        this.project = project;
    }

    public String getTarget() {
        return target;
    }

    public void setTarget(String target) {
        this.target = target;
    }

    public List<String> getOutput() {
        return output;
    }

    public void setOutput(List<String> output) {
        this.output = output;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        if (project != null) {
            map.put("project", project);
        }
        if (target != null) {
            map.put("target", target);
        }
        map.put("output", output);
        return map;
    }
}
