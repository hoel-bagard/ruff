"""Fixtures for the errors E301, E302, E303, E304, E305 and E306.

Since these errors are about new lines, each test starts with either "No error" or "# E30X".
Each test's end is signaled by a "# end" line.

There should be no E30X error outside of a test's bound.
"""


# No error
class Class:
    pass
# end
