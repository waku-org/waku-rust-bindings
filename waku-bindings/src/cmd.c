
/*
  This file is needed to avoid errors like the following when linking the waku-sys lib crate:
  <<undefined reference to `cmdCount'>>
  and
  <<undefined reference to `cmdLine'>>
*/

#include <stdio.h>

int cmdCount = 0;
char** cmdLine = NULL;

