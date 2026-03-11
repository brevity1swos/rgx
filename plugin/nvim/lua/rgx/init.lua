local M = {}

M.config = {
  cmd = "rgx",
  width = 0.8,
  height = 0.8,
  border = "rounded",
}

function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})

  vim.api.nvim_create_user_command("Rgx", function(cmd)
    M.open(cmd.args, cmd.range ~= 0)
  end, { nargs = "?", range = true })
end

function M.open(pattern, has_range)
  local args = {}

  -- If visual selection, pass as --text
  if has_range then
    local lines = vim.api.nvim_buf_get_lines(0, vim.fn.line("'<") - 1, vim.fn.line("'>"), false)
    local text = table.concat(lines, "\n")
    if text ~= "" then
      table.insert(args, "--text")
      table.insert(args, text)
    end
  end

  -- If pattern argument provided, pass it
  if pattern and pattern ~= "" then
    table.insert(args, pattern)
  end

  local cmd = M.config.cmd
  for _, arg in ipairs(args) do
    cmd = cmd .. " " .. vim.fn.shellescape(arg)
  end

  -- Calculate floating window dimensions
  local ui = vim.api.nvim_list_uis()[1]
  local width = math.floor(ui.width * M.config.width)
  local height = math.floor(ui.height * M.config.height)
  local col = math.floor((ui.width - width) / 2)
  local row = math.floor((ui.height - height) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    col = col,
    row = row,
    style = "minimal",
    border = M.config.border,
    title = " rgx ",
    title_pos = "center",
  })

  vim.fn.termopen(cmd, {
    on_exit = function()
      if vim.api.nvim_win_is_valid(win) then
        vim.api.nvim_win_close(win, true)
      end
      if vim.api.nvim_buf_is_valid(buf) then
        vim.api.nvim_buf_delete(buf, { force = true })
      end
    end,
  })

  vim.cmd("startinsert")
end

return M
