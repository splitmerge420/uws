import os
import json
import logging

logging.basicConfig(level=logging.INFO)

class CapabilityAudit:
    def __init__(self, directory):
        self.directory = directory
        self.report = {}

    def audit(self):
        """Perform an audit of the system capabilities in the specified directory."""
        logging.info(f'Auditing directory: {self.directory}')
        for root, dirs, files in os.walk(self.directory):
            for file in files:
                if file.endswith('.json'):
                    self.process_file(os.path.join(root, file))
        self.save_report()

    def process_file(self, file_path):
        """Process a JSON file to extract capability data."""
        with open(file_path, 'r') as f:
            try:
                data = json.load(f)
                # Assuming data format contains 'capabilities' key
                if 'capabilities' in data:
                    capabilities = data['capabilities']
                    self.report[file_path] = capabilities
                else:
                    logging.warning(f'No capabilities found in {file_path}')
            except json.JSONDecodeError:
                logging.error(f'Invalid JSON in file: {file_path}')

    def save_report(self):
        """Save the audit report to a JSON file."""
        report_file = os.path.join(self.directory, 'capability_audit_report.json')
        with open(report_file, 'w') as f:
            json.dump(self.report, f, indent=4)
        logging.info(f'Audit report saved to {report_file}')

if __name__ == '__main__':
    directory_to_audit = '.'  # Default to current directory
    audit_tool = CapabilityAudit(directory_to_audit)
    audit_tool.audit()