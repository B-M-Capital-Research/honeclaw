import os
import re

# We will read each file, find log_message_xxx calls, and add `None` as the last argument.
# To handle nested parens, we need a simple parens-matching parser.
def process_file(path):
    with open(path, 'r') as f:
        text = f.read()

    out = []
    i = 0
    changed = False
    methods = ["log_message_received", "log_message_step", "log_message_finished", "log_message_failed"]
    
    while i < len(text):
        found = False
        for m in methods:
            if text.startswith(m, i):
                # find the opening parenthesis
                idx = i + len(m)
                while idx < len(text) and text[idx] in ' \n\t':
                    idx += 1
                if idx < len(text) and text[idx] == '(':
                    # now parse until matching ')'
                    open_parens = 1
                    curr = idx + 1
                    while curr < len(text) and open_parens > 0:
                        if text[curr] == '(':
                            open_parens += 1
                        elif text[curr] == ')':
                            open_parens -= 1
                        curr += 1
                    
                    if open_parens == 0:
                        # curr is exactly after the closing paren
                        # The arguments are between idx+1 and curr-2
                        args_str = text[idx+1:curr-1]
                        # We append `, None` before the closing paren
                        # Wait, what if it already has None?
                        # We don't want to double append.
                        out.append(text[i:curr-1] + ", None)")
                        i = curr
                        found = True
                        changed = True
                        break
        if not found:
            out.append(text[i])
            i += 1

    if changed:
        with open(path, 'w') as f:
            f.write("".join(out))

for root, dirs, files in os.walk('.'):
    if 'target' in root: continue
    for f in files:
        if f.endswith('.rs'):
            process_file(os.path.join(root, f))

