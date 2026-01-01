import yaml
import template

# Test raw helper with multiline YAML content
content = template.execute("""{{raw value}}""", {value = yaml.encode(_val)})

_val = {
    name = "test"
    config = {
        timeout = "5m"
        retry = 3
    }
}
