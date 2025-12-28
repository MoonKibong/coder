package com.softbase.xframe5.codegen.model;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Database schema input for code generation.
 */
public class SchemaInput {

    private String table;
    private String schema;
    private List<SchemaColumn> columns = new ArrayList<>();
    private List<String> primaryKeys = new ArrayList<>();
    private List<ForeignKey> foreignKeys = new ArrayList<>();

    public SchemaInput() {
    }

    public SchemaInput(String table) {
        this.table = table;
    }

    public String getTable() {
        return table;
    }

    public void setTable(String table) {
        this.table = table;
    }

    public String getSchema() {
        return schema;
    }

    public void setSchema(String schema) {
        this.schema = schema;
    }

    public List<SchemaColumn> getColumns() {
        return columns;
    }

    public void setColumns(List<SchemaColumn> columns) {
        this.columns = columns;
    }

    public void addColumn(SchemaColumn column) {
        this.columns.add(column);
    }

    public List<String> getPrimaryKeys() {
        return primaryKeys;
    }

    public void setPrimaryKeys(List<String> primaryKeys) {
        this.primaryKeys = primaryKeys;
    }

    public List<ForeignKey> getForeignKeys() {
        return foreignKeys;
    }

    public void setForeignKeys(List<ForeignKey> foreignKeys) {
        this.foreignKeys = foreignKeys;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("type", "db_schema");
        map.put("table", table);
        if (schema != null) {
            map.put("schema", schema);
        }

        List<Map<String, Object>> columnMaps = new ArrayList<>();
        for (SchemaColumn col : columns) {
            columnMaps.add(col.toMap());
        }
        map.put("columns", columnMaps);
        map.put("primary_keys", primaryKeys);

        List<Map<String, Object>> fkMaps = new ArrayList<>();
        for (ForeignKey fk : foreignKeys) {
            fkMaps.add(fk.toMap());
        }
        map.put("foreign_keys", fkMaps);

        return map;
    }

    /**
     * Schema column definition.
     */
    public static class SchemaColumn {
        private String name;
        private String columnType;
        private boolean nullable = true;
        private boolean pk = false;
        private String defaultValue;
        private String comment;

        public SchemaColumn() {
        }

        public SchemaColumn(String name, String columnType) {
            this.name = name;
            this.columnType = columnType;
        }

        public String getName() {
            return name;
        }

        public void setName(String name) {
            this.name = name;
        }

        public String getColumnType() {
            return columnType;
        }

        public void setColumnType(String columnType) {
            this.columnType = columnType;
        }

        public boolean isNullable() {
            return nullable;
        }

        public void setNullable(boolean nullable) {
            this.nullable = nullable;
        }

        public boolean isPk() {
            return pk;
        }

        public void setPk(boolean pk) {
            this.pk = pk;
        }

        public String getDefaultValue() {
            return defaultValue;
        }

        public void setDefaultValue(String defaultValue) {
            this.defaultValue = defaultValue;
        }

        public String getComment() {
            return comment;
        }

        public void setComment(String comment) {
            this.comment = comment;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            map.put("name", name);
            map.put("column_type", columnType);
            map.put("nullable", nullable);
            map.put("pk", pk);
            if (defaultValue != null) {
                map.put("default", defaultValue);
            }
            if (comment != null) {
                map.put("comment", comment);
            }
            return map;
        }
    }

    /**
     * Foreign key definition.
     */
    public static class ForeignKey {
        private String column;
        private String refTable;
        private String refColumn;

        public ForeignKey() {
        }

        public ForeignKey(String column, String refTable, String refColumn) {
            this.column = column;
            this.refTable = refTable;
            this.refColumn = refColumn;
        }

        public String getColumn() {
            return column;
        }

        public void setColumn(String column) {
            this.column = column;
        }

        public String getRefTable() {
            return refTable;
        }

        public void setRefTable(String refTable) {
            this.refTable = refTable;
        }

        public String getRefColumn() {
            return refColumn;
        }

        public void setRefColumn(String refColumn) {
            this.refColumn = refColumn;
        }

        public Map<String, Object> toMap() {
            Map<String, Object> map = new HashMap<>();
            map.put("column", column);
            map.put("ref_table", refTable);
            map.put("ref_column", refColumn);
            return map;
        }
    }
}
