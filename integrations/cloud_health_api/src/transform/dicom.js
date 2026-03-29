const dicomParser = require('dicom-parser');

class DicomTransformer {
  constructor(options = {}){ this.options = options }

  // If given a Buffer containing DICOM bytes, try to parse using dicom-parser and return a JSON summary.
  transformBufferToJson(buffer){
    if(!buffer) throw new Error('No buffer provided');
    try{
      const byteArray = new Uint8Array(buffer);
      const dataSet = dicomParser.parseDicom(byteArray);
      // extract a few common tags for metadata (PatientName, PatientID, StudyDate)
      const patientName = dataSet.string('x00100010') || undefined;
      const patientId = dataSet.string('x00100020') || undefined;
      const studyDate = dataSet.string('x00080020') || undefined;
      return { dicomMetadata: { patientName, patientId, studyDate }, parsed: true };
    }catch(err){
      // Fallback: return minimal wrapper
      return { dicomMetadata: { raw: Buffer.isBuffer(buffer) ? buffer.toString('base64') : String(buffer) }, parsed: false, error: err.message };
    }
  }

  // Minimal validation: ensure dicomMetadata exists
  validateDicomJson(json){
    if(!json || !json.dicomMetadata) return { valid: false, reason: 'missing dicomMetadata' };
    return { valid: true };
  }
}

module.exports = { DicomTransformer };
