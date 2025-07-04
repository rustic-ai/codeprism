# Issue #174: Documentation Update Plan for Rust-SDK Implementation

## Problem Statement

The documentation currently references the legacy `codeprism-mcp` implementation, but the project has migrated to a new `codeprism-mcp-server` built on the official Rust MCP SDK. All documentation needs to be updated to reflect:

1. New binary name and command-line interface
2. Changed configuration system (profiles vs direct arguments)
3. Different environment variable structure
4. New installation and setup procedures
5. Updated API schemas and tool specifications

## Analysis of Required Changes

### Major Documentation Files Requiring Updates

1. **README.md**: Main project documentation with installation and quick start
2. **docs/GETTING_STARTED.md**: Detailed setup instructions for MCP clients
3. **docs/API.md**: Tool schemas and parameter documentation
4. **docs/PRISM-MCP-SERVER-DESCRIPTION.md**: Server capabilities description
5. **docs/DEVELOPER.md**: Development setup instructions
6. **docs/MCP-DOCUMENTATION.md**: MCP protocol implementation details

### Key Changes Required

#### 1. Binary and Installation Changes
- **Old**: `codeprism-mcp` binary
- **New**: `codeprism-mcp-server` binary
- **CLI Interface Change**: 
  - Old: `codeprism-mcp /path/to/repo`
  - New: `codeprism-mcp-server --profile development`

#### 2. Configuration System Changes
- **Old**: Direct repository path arguments + REPOSITORY_PATH env var
- **New**: Configuration profiles (development/production/enterprise) + CODEPRISM_* env vars

#### 3. Environment Variables Changes
- **Old**: `REPOSITORY_PATH`, `RUST_LOG`
- **New**: `CODEPRISM_PROFILE`, `CODEPRISM_MEMORY_LIMIT_MB`, `CODEPRISM_BATCH_SIZE`, etc.

#### 4. MCP Client Configuration Changes
- **Command Path**: Update all client configs to use new binary name
- **Arguments**: Remove repository path arguments 
- **Environment**: Update to use new environment variable structure

## Implementation Plan

### Phase 1: Core Documentation Updates

#### 1.1 Update README.md
- [ ] Change installation instructions to reference `codeprism-mcp-server`
- [ ] Update quick start commands and CLI examples
- [ ] Update MCP client configuration examples (Claude, Cursor)
- [ ] Update build instructions and verification commands
- [ ] Update performance metrics and capability descriptions

#### 1.2 Update GETTING_STARTED.md
- [ ] Replace all binary references: `codeprism-mcp` → `codeprism-mcp-server`
- [ ] Update command-line interface documentation
- [ ] Rewrite MCP client configuration sections:
  - Claude Desktop configuration
  - Cursor configuration  
  - VS Code configuration
- [ ] Update environment variable documentation
- [ ] Revise troubleshooting section for new architecture

#### 1.3 Update API Documentation
- [ ] Review and update tool parameter schemas
- [ ] Document new configuration options and profiles
- [ ] Update error response formats if changed
- [ ] Verify tool capability descriptions match implementation

### Phase 2: Technical Documentation Updates

#### 2.1 Update DEVELOPER.md
- [ ] Update development setup instructions
- [ ] Change build commands and binary references
- [ ] Update testing procedures
- [ ] Revise contribution guidelines for new architecture

#### 2.2 Update PRISM-MCP-SERVER-DESCRIPTION.md
- [ ] Update server capabilities description
- [ ] Document new configuration system
- [ ] Update performance characteristics
- [ ] Revise architecture description

#### 2.3 Update MCP-DOCUMENTATION.md
- [ ] Document rust-sdk based implementation
- [ ] Update protocol compliance information
- [ ] Revise tool registration and discovery

### Phase 3: Configuration and Setup Documentation

#### 3.1 Create Configuration Guide
- [ ] Document configuration profiles (development, production, enterprise)
- [ ] Create environment variable reference
- [ ] Provide configuration file examples (TOML, YAML, JSON)
- [ ] Document profile customization and validation

#### 3.2 Update Deployment Documentation
- [ ] Update Docker configurations if any
- [ ] Revise production deployment guides
- [ ] Update monitoring and logging setup
- [ ] Document new caching and security configurations

### Phase 4: Migration and Compatibility

#### 4.1 Create Migration Guide
- [ ] Document migration path from legacy to new implementation
- [ ] Provide compatibility notes and breaking changes
- [ ] Create automated migration scripts or tools if needed
- [ ] Document rollback procedures

#### 4.2 Update Tool and Resource Documentation
- [ ] Verify all 26 tools are documented correctly
- [ ] Update resource descriptions and schemas
- [ ] Check prompt examples and use cases
- [ ] Update performance benchmarks

## Specific Configuration Examples

### New MCP Client Configurations

#### Claude Desktop (New Format)
```json
{
  "mcpServers": {
    "codeprism": {
      "command": "/path/to/codeprism-mcp-server",
      "env": {
        "CODEPRISM_PROFILE": "development",
        "REPOSITORY_PATH": "/path/to/repository"
      }
    }
  }
}
```

#### Cursor (New Format) 
```json
{
  "mcpServers": {
    "codeprism": {
      "command": "/path/to/codeprism-mcp-server",
      "env": {
        "CODEPRISM_PROFILE": "development",
        "REPOSITORY_PATH": "."
      }
    }
  }
}
```

**Note**: Need to verify if REPOSITORY_PATH is still supported or if repository initialization happens differently.

### New Environment Variables

Replace legacy environment variables with new CODEPRISM_* prefixed ones:

```bash
# Legacy
export REPOSITORY_PATH=/path/to/repo
export RUST_LOG=info

# New
export CODEPRISM_PROFILE=development
export CODEPRISM_MEMORY_LIMIT_MB=1024
export CODEPRISM_BATCH_SIZE=10
export REPOSITORY_PATH=/path/to/repo  # If still supported
```

## Testing and Validation

### Documentation Testing Plan
1. **Installation Verification**: Test all installation methods on clean systems
2. **Client Configuration Testing**: Verify all MCP client configs work correctly
3. **Example Validation**: Run all code examples and verify they work
4. **Link Checking**: Ensure all internal and external links are valid
5. **Migration Testing**: Test migration procedures on real setups

### Validation Checklist
- [ ] All binary references updated to `codeprism-mcp-server`
- [ ] All command-line examples use new CLI interface
- [ ] All MCP client configurations use new format
- [ ] All environment variables use new naming
- [ ] No references to legacy implementation remain
- [ ] All examples tested and verified working
- [ ] Migration guide tested with real users

## Success Criteria

1. **Complete Migration**: No references to legacy `codeprism-mcp` remain
2. **Functional Documentation**: All setup procedures work on fresh installations
3. **Client Compatibility**: All major MCP clients work with new configuration
4. **Migration Support**: Clear path from legacy to new implementation
5. **Developer Experience**: Development setup works smoothly with new architecture

## Timeline and Dependencies

**Dependencies:**
- Verify final repository initialization approach in new server
- Confirm all environment variables and configuration options
- Test actual MCP client integration with new server

**Estimated Effort:**
- Phase 1: 2-3 days (core documentation)
- Phase 2: 1-2 days (technical documentation)  
- Phase 3: 1 day (configuration documentation)
- Phase 4: 1 day (migration and testing)

**Total Estimated Time**: 5-7 days of focused documentation work

## Follow-up Actions

After documentation updates:
1. Update issue #175 (Deprecate legacy crate) with migration guide
2. Support issue #176 (Remove legacy crate) with clean documentation
3. Create user communication about the migration
4. Update any external references or tutorials 