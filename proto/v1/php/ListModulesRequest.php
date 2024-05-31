<?php
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: proto/v1/api.proto

use Google\Protobuf\Internal\GPBType;
use Google\Protobuf\Internal\RepeatedField;
use Google\Protobuf\Internal\GPBUtil;

/**
 * `POST /api/v1/modules:`
 * Return paginated list of all modules.
 *
 * Generated from protobuf message <code>ListModulesRequest</code>
 */
class ListModulesRequest extends \Google\Protobuf\Internal\Message
{
    /**
     * Generated from protobuf field <code>.Pagination pagination = 1;</code>
     */
    protected $pagination = null;
    /**
     * Generated from protobuf field <code>.Sort sort = 2;</code>
     */
    protected $sort = null;

    /**
     * Constructor.
     *
     * @param array $data {
     *     Optional. Data for populating the Message object.
     *
     *     @type \Pagination $pagination
     *     @type \Sort $sort
     * }
     */
    public function __construct($data = NULL) {
        \GPBMetadata\Proto\V1\Api::initOnce();
        parent::__construct($data);
    }

    /**
     * Generated from protobuf field <code>.Pagination pagination = 1;</code>
     * @return \Pagination|null
     */
    public function getPagination()
    {
        return isset($this->pagination) ? $this->pagination : null;
    }

    public function hasPagination()
    {
        return isset($this->pagination);
    }

    public function clearPagination()
    {
        unset($this->pagination);
    }

    /**
     * Generated from protobuf field <code>.Pagination pagination = 1;</code>
     * @param \Pagination $var
     * @return $this
     */
    public function setPagination($var)
    {
        GPBUtil::checkMessage($var, \Pagination::class);
        $this->pagination = $var;

        return $this;
    }

    /**
     * Generated from protobuf field <code>.Sort sort = 2;</code>
     * @return \Sort|null
     */
    public function getSort()
    {
        return isset($this->sort) ? $this->sort : null;
    }

    public function hasSort()
    {
        return isset($this->sort);
    }

    public function clearSort()
    {
        unset($this->sort);
    }

    /**
     * Generated from protobuf field <code>.Sort sort = 2;</code>
     * @param \Sort $var
     * @return $this
     */
    public function setSort($var)
    {
        GPBUtil::checkMessage($var, \Sort::class);
        $this->sort = $var;

        return $this;
    }

}
