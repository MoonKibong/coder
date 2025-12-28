package com.softbase.xframe5.codegen;

import org.eclipse.jface.resource.ImageDescriptor;
import org.eclipse.ui.plugin.AbstractUIPlugin;
import org.osgi.framework.BundleContext;

/**
 * Enterprise Code Generator Plugin Activator
 *
 * This plugin is intentionally "dumb" - it knows nothing about LLM models,
 * prompts, or AI configuration. It only knows:
 * - Input types (db-schema, query-sample, natural-language)
 * - Server endpoint
 * - Project context
 */
public class Activator extends AbstractUIPlugin {

    public static final String PLUGIN_ID = "com.softbase.xframe5.codegen";

    private static Activator plugin;

    public Activator() {
    }

    @Override
    public void start(BundleContext context) throws Exception {
        super.start(context);
        plugin = this;
    }

    @Override
    public void stop(BundleContext context) throws Exception {
        plugin = null;
        super.stop(context);
    }

    public static Activator getDefault() {
        return plugin;
    }

    public static ImageDescriptor getImageDescriptor(String path) {
        return imageDescriptorFromPlugin(PLUGIN_ID, path);
    }
}
