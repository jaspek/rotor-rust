"""Patch /selfie/tools/rotor.c with the professor's profiling counters:
count invocations of the dedup question (new_line / find_equal_line),
the list-walk comparisons, and the hits. Also restore reuse_lines = 1
(the image's source was left at 0 by the CSE-off experiment build)."""
import io

p = '/selfie/tools/rotor.c'
s = io.open(p, encoding='utf-8', errors='replace').read()

# 1) restore reuse and declare counters
old = "uint64_t reuse_lines = 0; // flag for reusing lines"
if old not in s:
    old = "uint64_t reuse_lines = 1; // flag for reusing lines"
assert old in s, "reuse_lines decl not found"
s = s.replace(old, """uint64_t reuse_lines = 1; // flag for reusing lines

uint64_t profile_new_line_calls = 0;
uint64_t profile_find_calls = 0;
uint64_t profile_comparisons = 0;
uint64_t profile_hits = 0;""")

# 2) find_equal_line instrumentation
old = """uint64_t* find_equal_line(uint64_t* line) {
  uint64_t* pred_line;

  pred_line = last_line;

  while (pred_line) {
    if (are_lines_equal(pred_line, line))
      return pred_line;

    pred_line = get_pred(pred_line);
  }

  return UNUSED;
}"""
assert old in s, "find_equal_line not found"
s = s.replace(old, """uint64_t* find_equal_line(uint64_t* line) {
  uint64_t* pred_line;

  profile_find_calls = profile_find_calls + 1;

  pred_line = last_line;

  while (pred_line) {
    profile_comparisons = profile_comparisons + 1;

    if (are_lines_equal(pred_line, line)) {
      profile_hits = profile_hits + 1;

      return pred_line;
    }

    pred_line = get_pred(pred_line);
  }

  return UNUSED;
}""")

# 3) new_line invocation counter
old = """uint64_t* new_line(char* op, uint64_t* sid, uint64_t* arg1, uint64_t* arg2, uint64_t* arg3, char* comment) {
  uint64_t* new_line;
  uint64_t* old_line;"""
assert old in s, "new_line not found"
s = s.replace(old, """uint64_t* new_line(char* op, uint64_t* sid, uint64_t* arg1, uint64_t* arg2, uint64_t* arg3, char* comment) {
  uint64_t* new_line;
  uint64_t* old_line;

  profile_new_line_calls = profile_new_line_calls + 1;""")

# 4) print the counters next to the existing stats line
old = 'printf("%s: %lu lines of model formulae generated\\n", selfie_name, number_of_lines);'
assert old in s, "stats printf not found"
s = s.replace(old, old + """
  printf("%s: PROFILE new_line calls: %lu\\n", selfie_name, profile_new_line_calls);
  printf("%s: PROFILE find_equal_line calls: %lu\\n", selfie_name, profile_find_calls);
  printf("%s: PROFILE list comparisons: %lu\\n", selfie_name, profile_comparisons);
  printf("%s: PROFILE hits (lines reused): %lu\\n", selfie_name, profile_hits);""")

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print("rotor.c instrumented")
