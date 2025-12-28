package com.softbase.xframe5.codegen.client;

/**
 * Exception thrown when communication with the agent server fails.
 */
public class AgentClientException extends Exception {

    private static final long serialVersionUID = 1L;

    public AgentClientException(String message) {
        super(message);
    }

    public AgentClientException(String message, Throwable cause) {
        super(message, cause);
    }
}
