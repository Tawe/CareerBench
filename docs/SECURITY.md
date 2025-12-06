# Security Guide

## Protecting Sensitive Data

### What's Protected

The following sensitive data is stored locally and should **NEVER** be committed to git:

1. **Database** (`src-tauri/.careerbench/careerbench.db`)
   - Contains: User profile data, jobs, applications, AI settings, API keys
   - **Status**: ✅ Protected by `.gitignore`

2. **Log Files** (`src-tauri/.careerbench/careerbench.log`)
   - Contains: Application logs, may contain error messages
   - **Status**: ✅ Protected by `.gitignore`

3. **API Keys**
   - Stored in: Database (`ai_settings` table)
   - **Status**: ✅ Protected (database is ignored)

4. **Model Files** (`*.gguf`)
   - Large model files (~2.3GB)
   - **Status**: ✅ Protected by `.gitignore`

### Before Pushing to GitHub

**Always check** what you're committing:

```bash
# See what files are staged
git status

# See what's in your commits
git diff --cached

# Check for any database or log files
git ls-files | grep -E "(\.db|\.log|careerbench)"
```

### If You Accidentally Committed Sensitive Data

1. **If you haven't pushed yet:**
   ```bash
   # Remove from staging
   git reset HEAD <file>
   
   # Add to .gitignore (already done)
   # Commit the .gitignore change
   git add .gitignore
   git commit -m "Add .careerbench to gitignore"
   ```

2. **If you already pushed:**
   - **Immediately rotate your API keys** in OpenAI/Anthropic
   - Consider using `git filter-branch` or BFG Repo-Cleaner to remove from history
   - Or start fresh with a new repository

3. **Check your GitHub repository:**
   - Go to your repo settings
   - Check if the database file is visible
   - If it is, delete the repository and create a new one (or clean history)

### Current Protection Status

✅ **Database files** - Ignored (`.careerbench/`, `*.db`)  
✅ **Log files** - Ignored (`*.log`)  
✅ **API keys** - Stored in ignored database  
✅ **Model files** - Ignored (`*.gguf`)  
✅ **Environment files** - Ignored (`.env*`)

### Best Practices

1. **Never commit:**
   - Database files (`.db`)
   - Log files (`.log`)
   - API keys or secrets
   - Personal data
   - Large model files

2. **Always review before pushing:**
   ```bash
   git status
   git diff
   ```

3. **Use environment variables for development:**
   - Consider using `.env.local` for development API keys (already ignored)

4. **Rotate keys if exposed:**
   - If you suspect your API key was exposed, immediately rotate it
   - Check API usage logs for unauthorized access

