import sub.sub1 as sub1
import sub as sub
import .sub.sub2 as sub2

sub_in_subpkg = sub.sub
sub1_in_subpkg = sub1.sub1

sub_in_sub2_in_subpkg = sub2.sub_in_sub2
sub1_in_sub2_in_subpkg = sub2.sub1_in_sub2
