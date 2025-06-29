"""
Basic test to verify the testing infrastructure works.
"""

import pytest


def test_basic():
    """Basic test to verify pytest works."""
    assert True


class TestBasic:
    """Basic test class."""
    
    def test_class_based(self):
        """Test class-based test."""
        assert 1 + 1 == 2