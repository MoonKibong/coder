package com.softbase.xframe5.codegen.preferences;

/**
 * Constants for plugin preferences.
 *
 * Note: These preferences are for plugin configuration only.
 * They do NOT include any LLM-related settings (model, temperature, etc.)
 * because the plugin is intentionally "dumb" about LLM details.
 */
public class PreferenceConstants {

    /**
     * Agent server endpoint URL.
     */
    public static final String P_SERVER_ENDPOINT = "serverEndpoint";

    /**
     * Default server endpoint.
     */
    public static final String DEFAULT_SERVER_ENDPOINT = "http://localhost:3000";

    /**
     * Request timeout in seconds.
     */
    public static final String P_TIMEOUT = "timeout";

    /**
     * Default timeout (120 seconds for LLM generation).
     */
    public static final int DEFAULT_TIMEOUT = 120;

    /**
     * Company ID for company-specific rules.
     */
    public static final String P_COMPANY_ID = "companyId";

    /**
     * Output language preference.
     */
    public static final String P_LANGUAGE = "language";

    /**
     * Default language.
     */
    public static final String DEFAULT_LANGUAGE = "ko";

    /**
     * Strict mode preference.
     */
    public static final String P_STRICT_MODE = "strictMode";

    /**
     * Default views folder name.
     */
    public static final String P_VIEWS_FOLDER = "viewsFolder";

    /**
     * Default views folder.
     */
    public static final String DEFAULT_VIEWS_FOLDER = "views";

    /**
     * Default scripts folder name.
     */
    public static final String P_SCRIPTS_FOLDER = "scriptsFolder";

    /**
     * Default scripts folder.
     */
    public static final String DEFAULT_SCRIPTS_FOLDER = "scripts";

    /**
     * Spring base package preference.
     */
    public static final String P_SPRING_PACKAGE = "springPackage";

    /**
     * Default Spring base package.
     */
    public static final String DEFAULT_SPRING_PACKAGE = "com.company.project";

    private PreferenceConstants() {
        // Prevent instantiation
    }
}
