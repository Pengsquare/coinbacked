from jinja2 import Environment, FileSystemLoader
env = Environment(loader=FileSystemLoader('src/html'))
template = env.get_template('index.html')
output_from_parsed_template = template.render()
print(output_from_parsed_template)
