/**
 * Position of a node in source markdown: [start_line, start_col, end_line, end_col], 1-based.
 */
export type NodePosition = [number, number, number, number];

export type TableAlignment = 'none' | 'left' | 'center' | 'right';

export interface BaseAstNode {
  type: string;
  pos: NodePosition;
  children?: AstNode[];
}

export interface DocumentNode extends BaseAstNode {
  type: 'document';
  children: AstNode[];
}

export interface ParagraphNode extends BaseAstNode {
  type: 'paragraph';
  children: AstNode[];
}

export interface HeadingNode extends BaseAstNode {
  type: 'heading';
  level: 1 | 2 | 3 | 4 | 5 | 6;
  children: AstNode[];
}

export interface BlockQuoteNode extends BaseAstNode {
  type: 'block_quote';
  children: AstNode[];
}

export interface ListNode extends BaseAstNode {
  type: 'list';
  ordered: boolean;
  start?: number;
  tight: boolean;
  children: AstNode[];
}

export interface ListItemNode extends BaseAstNode {
  type: 'list_item';
  children: AstNode[];
}

export interface TaskItemNode extends BaseAstNode {
  type: 'task_item';
  checked: boolean;
  children: AstNode[];
}

export interface CodeBlockNode extends BaseAstNode {
  type: 'code_block';
  language?: string;
  value: string;
}

export interface ThematicBreakNode extends BaseAstNode {
  type: 'thematic_break';
}

export interface TableNode extends BaseAstNode {
  type: 'table';
  alignments: TableAlignment[];
  children: AstNode[];
}

export interface TableRowNode extends BaseAstNode {
  type: 'table_row';
  header: boolean;
  children: AstNode[];
}

export interface TableCellNode extends BaseAstNode {
  type: 'table_cell';
  children: AstNode[];
}

export interface TextNode extends BaseAstNode {
  type: 'text';
  value: string;
}

export interface EmphasisNode extends BaseAstNode {
  type: 'emphasis';
  children: AstNode[];
}

export interface StrongNode extends BaseAstNode {
  type: 'strong';
  children: AstNode[];
}

export interface StrikethroughNode extends BaseAstNode {
  type: 'strikethrough';
  children: AstNode[];
}

export interface CodeNode extends BaseAstNode {
  type: 'code';
  value: string;
}

export interface LinkNode extends BaseAstNode {
  type: 'link';
  url: string;
  title?: string;
  children: AstNode[];
}

export interface ImageNode extends BaseAstNode {
  type: 'image';
  url: string;
  title?: string;
  children: AstNode[];
}

export interface SoftBreakNode extends BaseAstNode {
  type: 'soft_break';
}

export interface LineBreakNode extends BaseAstNode {
  type: 'line_break';
}

export interface FootnoteDefinitionNode extends BaseAstNode {
  type: 'footnote_definition';
  name: string;
  children: AstNode[];
}

export interface FootnoteReferenceNode extends BaseAstNode {
  type: 'footnote_reference';
  name: string;
}

export interface MentionNode extends BaseAstNode {
  type: 'mention';
  username: string;
  text: string;
}

export interface TagNode extends BaseAstNode {
  type: 'tag';
  name: string;
  text: string;
}

export type AstNode =
  | DocumentNode
  | ParagraphNode
  | HeadingNode
  | BlockQuoteNode
  | ListNode
  | ListItemNode
  | TaskItemNode
  | CodeBlockNode
  | ThematicBreakNode
  | TableNode
  | TableRowNode
  | TableCellNode
  | TextNode
  | EmphasisNode
  | StrongNode
  | StrikethroughNode
  | CodeNode
  | LinkNode
  | ImageNode
  | SoftBreakNode
  | LineBreakNode
  | FootnoteDefinitionNode
  | FootnoteReferenceNode
  | MentionNode
  | TagNode;

export interface AstDocument {
  schema: number;
  root: DocumentNode;
}

/**
 * Converts Markdown string or buffer to safe HTML using generic CommonMark + GFM pipeline.
 */
export function toHtml(input: string | Uint8Array): string;
export function to_html(input: string | Uint8Array): string;

/**
 * Converts Markdown string or buffer to AST JSON string using generic CommonMark + GFM pipeline.
 */
export function toAst(input: string | Uint8Array): string;
export function to_ast(input: string | Uint8Array): string;

/**
 * Returns the current AST schema version (currently 1).
 */
export function astSchemaVersion(): number;
export function ast_schema_version(): number;

/**
 * AST JSON schema version constant.
 */
export const AST_SCHEMA_VERSION: number;

/**
 * markstone package version.
 */
export const version: string;

/**
 * Actos-specific extensions (@mentions and #tags).
 */
export namespace actos {
  /**
   * Converts Markdown string or buffer to safe HTML with Actos extensions (@mentions, #tags).
   */
  export function toHtml(input: string | Uint8Array): string;
  export function to_html(input: string | Uint8Array): string;

  /**
   * Converts Markdown string or buffer to AST JSON string with Actos extensions (@mentions, #tags).
   */
  export function toAst(input: string | Uint8Array): string;
  export function to_ast(input: string | Uint8Array): string;

  export function astSchemaVersion(): number;
  export function ast_schema_version(): number;

  export const AST_SCHEMA_VERSION: number;
  export const version: string;
}

export interface MarkstoneBinding {
  toHtml: typeof toHtml;
  to_html: typeof to_html;
  toAst: typeof toAst;
  to_ast: typeof to_ast;
  astSchemaVersion: typeof astSchemaVersion;
  ast_schema_version: typeof ast_schema_version;
  AST_SCHEMA_VERSION: typeof AST_SCHEMA_VERSION;
  version: typeof version;
  actos: typeof actos;
}

declare const markstone: MarkstoneBinding;
export default markstone;
