import some0.pkg1 as some00
import some1.pkg1 as some10
import ..some1.pkg1 as some11

Name1 = some00.Name  # some0.pkg1.name
Name2 = some10.Name  # some1.pkg1.name
Name3 = some11.Name  # some1.pkg1.name
