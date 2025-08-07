# changement

> Manage versioning and publishing for packages in your project

## Installation

Install globally via npm:

```bash
npm i -g changement
```

Or use in a Node.js project:

```bash
npm install changement
```

## Commands

| Command   | Description                                                           |
| :-------- | :-------------------------------------------------------------------- |
| `init`    | Initialize `changement` in a new project                              |
| `new`     | Create a new change for a package in your project                     |
| `version` | Apply all changes to your project and update the versions of packages |
| `publish` | Publish the packages to the registry                                  |
| `tag`     | Create the git tags for the current version of all packages           |

### new command

Create a new change for a package in your project.

```bash
change new -p example -m "Add new feature" -b minor
```

| Option            | Description                                      |
| :---------------- | :----------------------------------------------- |
| `-p`, `--package` | The name of the package to create the change for |
| `-m`, `--message` | The message for the change                       |
| `-b`, `--bump`    | The type of version bump (major, minor, patch)   |

New changes are stored in the `.changes` directory as a markdown file. This
markdown file has the following shape:

```md
---
"package-name": minor
---

Example description of the change
```

There can be multiple package names in the frontmatter of the markdown file
which signify that this change applies to multiple packages.

### version command

| Option     | Description                                                                 |
| :--------- | :-------------------------------------------------------------------------- |
| `--filter` | Filter packages to create changes for (e.g. `--filter=package-a,package-b`) |

### publish command

### tag command

| Option     | Description                                                                 |
| :--------- | :-------------------------------------------------------------------------- |
| `--filter` | Filter packages to create changes for (e.g. `--filter=package-a,package-b`) |

## Configuration

Configuration is stored in a `.changes/config.json` file. It has the following
structure:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "version": {
      "type": "number",
      "description": "The version of the changement configuration"
    },
    "ignore": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "Packages to ignore when running commands",
      "default": []
    }
  }
}
```
