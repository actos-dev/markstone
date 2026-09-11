# markstone

Python bindings for markstone: fast, safe Markdown-to-HTML and AST engine with Actos extensions.

## Installation

```bash
pip install markstone
```

## Usage

```python
import markstone

# Generic Markdown
html = markstone.to_html("# Hello world")
ast = markstone.to_ast("# Hello world")

# Actos extensions (@mentions and #tags)
actos_html = markstone.actos.to_html("Hello @alice and #rust")
actos_ast = markstone.actos.to_ast("Hello @alice and #rust")
```
